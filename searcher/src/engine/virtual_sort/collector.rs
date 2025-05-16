use corelib::{api, model::MetaSchema};
use std::cmp::Ordering;
use tantivy::collector::{Collector, SegmentCollector};
use tantivy::{DocAddress, DocId, Score, SegmentOrdinal, SegmentReader};
use tracing::debug;

use crate::engine::virtual_sort::eval::eval_program;
use crate::engine::virtual_sort::program::Program;

pub struct SortByVirtualFieldCollector<'a> {
    pub limit: usize,
    pub offset: usize,
    pub program: Program,
    pub schema: &'a MetaSchema,
}

pub struct VirtualFieldSegmentCollector {
    pub program: Program,
    pub segment_ordinal: SegmentOrdinal,
    pub field_readers: Vec<(usize, FieldReader)>,
    pub doc_ids: Vec<DocId>,
    pub max_docs: usize,
}

#[derive(Debug, PartialEq)]
pub struct ScoredDoc {
    pub sort_value: f32,
    pub doc: DocAddress,
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
        let mut field_readers = Vec::with_capacity(self.program.env.len());

        for (var_idx, var_name) in self.program.env.iter().enumerate() {
            let column = self.schema.get_column(var_name).map_err(|e| {
                tantivy::TantivyError::InvalidArgument(format!("Unknown column `{var_name}`: {e}"))
            })?;

            debug!(
                index = var_idx,
                column_name = %var_name,
                ?column.column_type,
                "Preparing fast field reader"
            );

            let reader = match column.column_type {
                api::MetaColumnType::DateTime => {
                    let col = segment.fast_fields().date(var_name)?;
                    FieldReader::Date(col)
                }
                api::MetaColumnType::Double => {
                    let col = segment.fast_fields().f64(var_name)?;
                    FieldReader::F64(col)
                }
                api::MetaColumnType::Bool => {
                    let col = segment.fast_fields().bool(var_name)?;
                    FieldReader::Bool(col)
                }
                other => {
                    return Err(tantivy::TantivyError::InvalidArgument(format!(
                        "Unsupported fast field type in virtual sort: {:?}",
                        other
                    )));
                }
            };

            field_readers.push((var_idx, reader));
        }

        Ok(VirtualFieldSegmentCollector {
            program: self.program.clone(),
            segment_ordinal,
            field_readers,
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

    fn collect(&mut self, doc_id: DocId, _score: Score) {
        self.doc_ids.push(doc_id);
    }

    fn harvest(self) -> Vec<ScoredDoc> {
        let mut results = Vec::with_capacity(self.doc_ids.len());
        let mut ctx = vec![0.0f32; self.program.env.len()];

        for doc_id in self.doc_ids {
            // читаем все значения из fastfield'ов
            for &(var_idx, ref reader) in &self.field_readers {
                ctx[var_idx] = reader.read_f32(doc_id);
            }

            if let Ok(score) = eval_program(&self.program, &ctx) {
                results.push(ScoredDoc {
                    sort_value: score,
                    doc: DocAddress::new(self.segment_ordinal, doc_id),
                });
            }
        }

        // отсекаем лишнее
        if results.len() > self.max_docs {
            results.select_nth_unstable_by(self.max_docs, ScoredDoc::cmp);
            results.truncate(self.max_docs);
        }

        results
    }
}

pub enum FieldReader {
    Date(tantivy::fastfield::Column<tantivy::DateTime>),
    F64(tantivy::fastfield::Column<f64>),
    Bool(tantivy::fastfield::Column<bool>),
}

impl FieldReader {
    pub fn read_f32(&self, doc_id: DocId) -> f32 {
        match self {
            FieldReader::Date(col) => col
                .values_for_doc(doc_id)
                .next()
                .map(|dt| dt.into_timestamp_millis() as f32)
                .unwrap_or(0.0),
            FieldReader::F64(col) => col.values_for_doc(doc_id).next().unwrap_or(0.0) as f32,
            FieldReader::Bool(col) => col
                .values_for_doc(doc_id)
                .next()
                .map(|b| if b { 1.0 } else { 0.0 })
                .unwrap_or(0.0),
        }
    }
}
