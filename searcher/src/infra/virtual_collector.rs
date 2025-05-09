use crate::infra::index::SearchIndex;
use crate::infra::online::evaluation::execute as eval_program;
use crate::infra::online::program::{OpCode, Program};
use corelib::model;
use std::cmp::Ordering;
use std::collections::HashMap;
use tantivy::collector::{Collector, SegmentCollector};
use tantivy::{DocAddress, DocId, Score, SegmentOrdinal, SegmentReader};

pub struct SortByVirtualFieldCollector<'a> {
    pub limit: usize,
    pub offset: usize,
    pub program: Program,
    pub schema: &'a model::MetaSchema,
}

pub struct VirtualFieldSegmentCollector {
    pub program: Program,
    pub field_map: Vec<(String, Box<dyn Fn(DocId) -> f64 + Send + Sync>)>,
    pub segment_ordinal: SegmentOrdinal,
    pub docs: Vec<ScoredDoc>,
    pub max_docs: usize,
}

#[derive(Debug, PartialEq)]
pub struct ScoredDoc {
    sort_value: f32,
    doc: DocAddress,
}

impl Eq for ScoredDoc {}

impl Ord for ScoredDoc {
    fn cmp(&self, other: &Self) -> Ordering {
        self.sort_value.total_cmp(&other.sort_value)
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
        segment: &SegmentReader,
    ) -> tantivy::Result<Self::Child> {
        let mut field_map = Vec::new();

        for op in &self.program.ops {
            if let OpCode::PushVariable(name) = op {
                let reader = segment.fast_fields().date(name)?;
                let reader_fn: Box<dyn Fn(DocId) -> f64 + Send + Sync> =
                    Box::new(move |doc_id: DocId| {
                        reader
                            .values_for_doc(doc_id)
                            .next()
                            .map(|dt| dt.into_timestamp_millis() as f64)
                            .unwrap_or(0.0)
                    });
                field_map.push((name.clone(), reader_fn));
            }
        }

        Ok(VirtualFieldSegmentCollector {
            program: self.program.clone(),
            field_map,
            segment_ordinal,
            docs: Vec::new(),
            max_docs: self.offset + self.limit,
        })
    }

    fn merge_fruits(
        &self,
        segment_fruits: Vec<<Self::Child as SegmentCollector>::Fruit>,
    ) -> tantivy::Result<Self::Fruit> {
        let mut docs: Vec<ScoredDoc> = segment_fruits.into_iter().flatten().collect();
        let cutoff = self.offset + self.limit;

        if cutoff < docs.len() {
            docs.select_nth_unstable_by(cutoff, ScoredDoc::cmp);
            docs[..cutoff].sort_unstable_by(ScoredDoc::cmp);
            Ok(docs[self.offset..cutoff]
                .iter()
                .map(|sd| (sd.sort_value, sd.doc))
                .collect())
        } else {
            docs.sort_unstable_by(ScoredDoc::cmp);
            Ok(docs
                .into_iter()
                .skip(self.offset)
                .map(|sd| (sd.sort_value, sd.doc))
                .collect())
        }
    }
}

impl SegmentCollector for VirtualFieldSegmentCollector {
    type Fruit = Vec<ScoredDoc>;

    fn collect(&mut self, doc: DocId, _score: Score) {
        let doc_addr = DocAddress::new(self.segment_ordinal, doc);

        let mut ctx = HashMap::with_capacity(self.field_map.len());
        for (name, reader_fn) in &self.field_map {
            let val = reader_fn(doc);
            ctx.insert(name.clone(), val);
        }

        if let Ok(value) = eval_program(&self.program, &ctx) {
            self.docs.push(ScoredDoc {
                sort_value: value as f32,
                doc: doc_addr,
            });
        }
    }

    fn harvest(mut self) -> Self::Fruit {
        if self.docs.len() > self.max_docs {
            self.docs
                .select_nth_unstable_by(self.max_docs, ScoredDoc::cmp);
            self.docs.truncate(self.max_docs);
        }
        self.docs
    }
}
