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
    pub segment_ordinal: SegmentOrdinal,
    pub docs: Vec<ScoredDoc>,
}

#[derive(Debug, PartialEq)]
pub struct ScoredDoc {
    sort_value: f32,
    doc: DocAddress,
}

impl Eq for ScoredDoc {}

impl Ord for ScoredDoc {
    // reverce order for heap
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
    type Fruit = Vec<(f32, DocAddress)>;
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
            segment_ordinal,
            docs: Vec::new(),
        })
    }

    fn merge_fruits(
        &self,
        segment_fruits: Vec<<Self::Child as SegmentCollector>::Fruit>,
    ) -> tantivy::Result<Self::Fruit> {
        let mut heap = BinaryHeap::<ScoredDoc>::with_capacity(self.limit + self.offset + 1);

        for scored_doc in segment_fruits.into_iter().flatten() {
            heap.push(scored_doc);
            if heap.len() > self.limit + self.offset {
                heap.pop();
            }
        }

        let results = heap
            .into_sorted_vec()
            .into_iter()
            .skip(self.offset)
            .take(self.limit)
            .map(|sd| (sd.sort_value, sd.doc))
            .collect();
        Ok(results)
    }
}

impl SegmentCollector for VirtualFieldSegmentCollector {
    type Fruit = Vec<ScoredDoc>;

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
            self.docs.push(ScoredDoc {
                sort_value: value as f32,
                doc: doc_addr,
            });
        }
    }

    fn harvest(self) -> Self::Fruit {
        self.docs
    }
}
