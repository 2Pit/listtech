use tantivy::schema::OwnedValue;

pub fn to_string_owned_value(value: &OwnedValue) -> String {
    match value {
        OwnedValue::Null => "null".to_string(),
        OwnedValue::Str(s) => format!("\"{}\"", s),
        OwnedValue::PreTokStr(p) => format!("\"{}\"", p.text),
        OwnedValue::U64(n) => n.to_string(),
        OwnedValue::I64(n) => n.to_string(),
        OwnedValue::F64(n) => n.to_string(),
        OwnedValue::Bool(b) => b.to_string(),
        OwnedValue::Date(dt) => dt.into_timestamp_millis().to_string(),
        OwnedValue::Facet(f) => format!("\"{}\"", f.to_path_string()),
        OwnedValue::Bytes(_) => "hex".to_string(),
        OwnedValue::Array(arr) => {
            let inner = arr
                .iter()
                .map(to_string_owned_value)
                .collect::<Vec<_>>()
                .join(", ");
            format!("[{}]", inner)
        }
        OwnedValue::Object(obj) => {
            let inner = obj
                .iter()
                .map(|(k, v)| format!("\"{}\": {}", k, to_string_owned_value(v)))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{{{}}}", inner)
        }
        OwnedValue::IpAddr(addr) => format!("\"{}\"", addr),
    }
}
