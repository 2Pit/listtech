use crate::domain::document::owned_val_as_f64;
use crate::infra::index::SearchIndex;
use crate::infra::online::evaluation::execute as eval_program;
use crate::infra::online::program::{OpCode, Program};
use anyhow::{Result, anyhow};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use tantivy::collector::{Collector, SegmentCollector};
use tantivy::schema::{Field, OwnedValue};
use tantivy::{DocAddress, DocId, Score, SegmentOrdinal, SegmentReader};

pub struct SortByVirtualFieldCollector<'a> {
    pub limit: usize,
    pub offset: usize,
    pub program: Program,
    pub index: &'a SearchIndex,
}

pub struct VirtualFieldSegmentCollector {
    pub program: Program,
    pub searcher: tantivy::Searcher,
    pub schema: corelib::model::MetaSchema,
    pub limit: usize,
    pub offset: usize,
    pub buffer: BinaryHeap<ScoredDoc>,
    pub segment_ordinal: SegmentOrdinal,
}

#[derive(Debug, PartialEq)]
struct ScoredDoc {
    sort_value: f64,
    doc: DocAddress,
}

impl Eq for ScoredDoc {}

impl Ord for ScoredDoc {
    fn cmp(&self, other: &Self) -> Ordering {
        other.sort_value.total_cmp(&self.sort_value)
    }
}

impl PartialOrd for ScoredDoc {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Collector for SortByVirtualFieldCollector<'a> {
    type Fruit = Vec<(f64, DocAddress)>;
    type Child = VirtualFieldSegmentCollector;

    fn requires_scoring(&self) -> bool {
        false
    }

    fn for_segment(
        &self,
        segment_ordinal: SegmentOrdinal,
        _segment: &SegmentReader,
    ) -> tantivy::Result<Self::Child> {
        let searcher = self.index.reader.searcher();
        Ok(VirtualFieldSegmentCollector {
            program: self.program.clone(),
            searcher,
            schema: self.index.schema.clone(),
            limit: self.limit,
            offset: self.offset,
            buffer: BinaryHeap::with_capacity(self.limit + self.offset),
            segment_ordinal,
        })
    }

    fn merge_fruits(
        &self,
        segment_fruits: Vec<<Self::Child as SegmentCollector>::Fruit>,
    ) -> tantivy::Result<Self::Fruit> {
        let mut combined: Vec<(f64, DocAddress)> = segment_fruits.into_iter().flatten().collect();
        combined.sort_by(|a, b| b.0.total_cmp(&a.0));
        combined.drain(0..self.offset.min(combined.len()));
        combined.truncate(self.limit);
        Ok(combined)
    }
}

impl SegmentCollector for VirtualFieldSegmentCollector {
    type Fruit = Vec<(f64, DocAddress)>;

    fn collect(&mut self, doc: DocId, _score: Score) {
        let doc_addr = DocAddress::new(self.segment_ordinal, doc);
        let doc_res: Result<HashMap<Field, OwnedValue>> =
            self.searcher.doc(doc_addr).map_err(|err| anyhow!(err));

        let Ok(doc) = doc_res else {
            return;
        };

        let ctx: Result<HashMap<String, f64>> = self
            .program
            .ops
            .iter()
            .filter_map(|op| match op {
                OpCode::PushVariable(name) => Some(name.clone()),
                _ => None,
            })
            .flat_map(|var| self.schema.get_idx(&var).ok().map(|field| (var, field)))
            .map(|(name, field)| {
                doc.get(&field)
                    .ok_or_else(|| anyhow!("Missing field {name}"))
                    .and_then(|v: &OwnedValue| owned_val_as_f64(v).map(|f64| (name, f64)))
            })
            .collect();

        let Ok(ctx) = ctx else {
            return;
        };

        if let Ok(value) = eval_program(&self.program, &ctx) {
            self.buffer.push(ScoredDoc {
                sort_value: value,
                doc: doc_addr,
            });
        }
    }

    fn harvest(self) -> Self::Fruit {
        self.buffer
            .into_sorted_vec()
            .into_iter()
            .map(|sd| (sd.sort_value, sd.doc))
            .collect()
    }
}
