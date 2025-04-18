use corelib::proto::searcher::{SearchField, search_field::FacetWrapper, search_field::Value};
use tantivy::schema::OwnedValue;

pub fn map_owned_value(field_name: &str, value: OwnedValue) -> SearchField {
    let value_enum = match value {
        OwnedValue::Str(s) => Value::StringValue(s),
        OwnedValue::PreTokStr(p) => Value::StringValue(p.text),
        OwnedValue::U64(n) => Value::UlongValue(n),
        OwnedValue::I64(n) => Value::LongValue(n),
        OwnedValue::F64(n) => Value::DoubleValue(n),
        OwnedValue::Bool(b) => Value::BoolValue(b),
        OwnedValue::Date(dt) => Value::TimestampMsValue(dt.into_timestamp_millis()),
        OwnedValue::Facet(f) => Value::FacetWrapper(FacetWrapper {
            facets: vec![f.to_string()],
        }),
        OwnedValue::Bytes(b) => Value::BytesValue(b.clone()),

        OwnedValue::Null | OwnedValue::Array(_) | OwnedValue::Object(_) | OwnedValue::IpAddr(_) => {
            return SearchField {
                name: field_name.to_string(),
                value: None,
            };
        }
    };

    SearchField {
        name: field_name.to_string(),
        value: Some(value_enum),
    }
}
