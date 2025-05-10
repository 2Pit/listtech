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
    pub doc_ids: Vec<DocId>,
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
            doc_ids: Vec::new(),
            max_docs: self.offset + self.limit,
        })
    }

    fn merge_fruits(
        &self,
        segment_fruits: Vec<<Self::Child as SegmentCollector>::Fruit>,
    ) -> tantivy::Result<Self::Fruit> {
        let mut scored_docs: Vec<ScoredDoc> = segment_fruits.into_iter().flatten().collect();
        let cutoff = self.offset + self.limit;

        if cutoff < scored_docs.len() {
            scored_docs.select_nth_unstable_by(cutoff, ScoredDoc::cmp);
            scored_docs[..cutoff].sort_unstable_by(ScoredDoc::cmp);
            Ok(scored_docs[self.offset..cutoff]
                .iter()
                .map(|sd| (sd.sort_value, sd.doc))
                .collect())
        } else {
            scored_docs.sort_unstable_by(ScoredDoc::cmp);
            Ok(scored_docs
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
        self.doc_ids.push(doc);
    }

    fn harvest(self) -> Self::Fruit {
        let mut docs = Vec::with_capacity(self.doc_ids.len());

        let mut ctx = HashMap::with_capacity(self.field_map.len());
        for &doc_id in &self.doc_ids {
            for (name, reader_fn) in &self.field_map {
                let val = reader_fn(doc_id);
                ctx.insert(name.clone(), val);
            }

            if let Ok(score) = eval_program(&self.program, &ctx) {
                docs.push(ScoredDoc {
                    sort_value: score as f32,
                    doc: DocAddress::new(self.segment_ordinal, doc_id),
                });
            }
            ctx.clear();
        }

        if docs.len() > self.max_docs {
            docs.select_nth_unstable_by(self.max_docs, ScoredDoc::cmp);
            docs.truncate(self.max_docs);
        }

        docs
    }
}
