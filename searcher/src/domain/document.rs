use crate::api;
use crate::api::SearchValue::*;
// use anyhow::{Ok, Result, anyhow};

use tantivy::schema::OwnedValue;

pub fn map_owned_value(field_name: &str, value: OwnedValue) -> api::SearchField {
    let value_enum = match value {
        OwnedValue::Str(s) => Str(s),
        OwnedValue::PreTokStr(p) => Str(p.text),
        OwnedValue::U64(n) => Ulong(n),
        OwnedValue::I64(n) => Long(n),
        OwnedValue::F64(n) => Double(n),
        OwnedValue::Bool(b) => Bool(b),
        OwnedValue::Date(dt) => DateTime(tantivy_datetime_to_iso(dt)),
        OwnedValue::Facet(f) => Tree(vec![f.to_string()]),
        OwnedValue::Bytes(b) => Bytes(b.clone()),

        // OwnedValue::Null | OwnedValue::Array(_) | OwnedValue::Object(_) | OwnedValue::IpAddr(_) =>
        _ => NullableBool(None),
    };

    api::SearchField {
        name: field_name.to_string(),
        value: value_enum,
    }
}

// pub fn owned_val_as_f32(value: &OwnedValue) -> Result<f32> {
//     let value_enum = match value {
//         OwnedValue::U64(n) => *n as f32,
//         OwnedValue::I64(n) => *n as f32,
//         OwnedValue::F64(n) => *n as f32,
//         OwnedValue::Bool(b) => {
//             if *b {
//                 1.0
//             } else {
//                 0.0
//             }
//         }
//         OwnedValue::Date(dt) => dt.into_timestamp_millis() as f32,

//         t => return Err(anyhow!("Cannot convert type to double {:?}", t)),
//     };

//     Ok(value_enum)
// }

fn tantivy_datetime_to_iso(dt: tantivy::DateTime) -> String {
    let micros = dt.into_timestamp_micros(); // Получаем i64 микросекунды
    let secs = micros / 1_000_000;
    let nanos = (micros % 1_000_000) * 1000; // остаток переводим в наносекунды

    let chrono_dt = chrono::DateTime::from_timestamp(secs, nanos as u32)
        .expect("Invalid timestamp")
        .naive_utc();

    chrono_dt.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string()
}
