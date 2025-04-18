use corelib::proto::indexer::Document;
use corelib::proto::indexer::indexable_field::Value;

/// Извлекает значение поля `asin` из protobuf-документа (если есть)
pub fn extract_asin(doc: &Document) -> Option<String> {
    doc.fields
        .iter()
        .find(|f| f.name == "asin")
        .and_then(|f| f.value.as_ref())
        .and_then(|v| match v {
            Value::StringValue(s) => Some(s.clone()),
            _ => None,
        })
}
