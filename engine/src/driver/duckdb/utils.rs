use crate::error::Result;
use base64::prelude::*;
use serde_json::json;

pub fn duckdb_value_to_json(value: duckdb::types::Value) -> Result<serde_json::Value> {
    let value = match value {
        duckdb::types::Value::Null => serde_json::Value::Null,
        duckdb::types::Value::Boolean(b) => b.into(),
        duckdb::types::Value::TinyInt(i) => i.into(),
        duckdb::types::Value::SmallInt(i) => i.into(),
        duckdb::types::Value::Int(i) => i.into(),
        duckdb::types::Value::BigInt(i) => i.into(),
        duckdb::types::Value::HugeInt(i) => json!(i),
        duckdb::types::Value::UTinyInt(i) => i.into(),
        duckdb::types::Value::USmallInt(i) => i.into(),
        duckdb::types::Value::UInt(i) => i.into(),
        duckdb::types::Value::UBigInt(i) => i.into(),
        duckdb::types::Value::Float(f) => json!(f),
        duckdb::types::Value::Double(f) => json!(f),
        duckdb::types::Value::Decimal(decimal) => {
            json!(decimal.to_string())
        }
        duckdb::types::Value::Timestamp(_, timestamp) => {
            json!(timestamp.to_string())
        }
        duckdb::types::Value::Text(s) => json!(s),
        duckdb::types::Value::Blob(vec) => {
            json!(BASE64_STANDARD.encode(&vec))
        }
        duckdb::types::Value::Date32(date) => {
            json!(date.to_string())
        }
        duckdb::types::Value::Time64(_, time) => {
            json!(time.to_string())
        }
        duckdb::types::Value::Interval {
            months,
            days,
            nanos,
        } => {
            json!({
                "months": months,
                "days": days,
                "nanos": nanos
            })
        }
        duckdb::types::Value::List(vec) => {
            let mut json_array = Vec::with_capacity(vec.len());
            for item in vec {
                json_array.push(duckdb_value_to_json(item)?);
            }
            json!(json_array)
        }
        duckdb::types::Value::Enum(e) => {
            json!(e.to_string())
        }
        duckdb::types::Value::Struct(ordered_map) => {
            let mut map = serde_json::Map::new();
            for (key, value) in ordered_map.iter() {
                map.insert(key.to_string(), duckdb_value_to_json(value.to_owned())?);
            }
            serde_json::Value::Object(map)
        }
        duckdb::types::Value::Array(vec) => {
            let mut json_array = Vec::with_capacity(vec.len());
            for item in vec {
                json_array.push(duckdb_value_to_json(item)?);
            }
            json!(json_array)
        }
        duckdb::types::Value::Map(ordered_map) => {
            // Convert map to JSON object
            let mut map = serde_json::Map::new();
            for (key, value) in ordered_map.iter() {
                let key_str = match duckdb_value_to_json(key.to_owned())? {
                    serde_json::Value::String(s) => s,
                    k => k.to_string(),
                };
                map.insert(key_str, duckdb_value_to_json(value.to_owned())?);
            }
            serde_json::Value::Object(map)
        }
        duckdb::types::Value::Union(value) => duckdb_value_to_json(*value)?,
    };

    Ok(value)
}
