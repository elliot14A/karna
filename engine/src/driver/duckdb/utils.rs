use crate::error::{DuckDBValueConversionSnafu, Error, Result};
use chrono::prelude::{DateTime, NaiveDate, NaiveTime};
use duckdb::types::{OrderedMap, TimeUnit, Value as DuckDBValue};

use serde_json::{Map, Number, Value as JsonValue};
use snafu::OptionExt;

fn duckdb_value_to_json_value(value: DuckDBValue) -> Result<JsonValue> {
    match value {
        // Basic scalar types
        DuckDBValue::Null => Ok(JsonValue::Null),
        DuckDBValue::Boolean(b) => Ok(JsonValue::Bool(b)),

        // Integer types
        DuckDBValue::TinyInt(i) => Ok(JsonValue::Number(i.into())),
        DuckDBValue::UTinyInt(i) => Ok(JsonValue::Number((i as i8).into())),
        DuckDBValue::SmallInt(i) => Ok(JsonValue::Number(i.into())),
        DuckDBValue::USmallInt(i) => Ok(JsonValue::Number((i as i16).into())),
        DuckDBValue::Int(i) => Ok(JsonValue::Number(i.into())),
        DuckDBValue::UInt(i) => Ok(JsonValue::Number((i as i32).into())),
        DuckDBValue::BigInt(i) => Ok(JsonValue::Number(i.into())),
        DuckDBValue::UBigInt(i) => Ok(JsonValue::Number((i as i64).into())),

        // Floating point numbers
        DuckDBValue::Float(f) => Ok(float_to_json(f.into())),
        DuckDBValue::Double(f) => Ok(float_to_json(f)),

        // String types
        DuckDBValue::Text(s) | DuckDBValue::Enum(s) => Ok(JsonValue::String(s)),

        // Binary data
        DuckDBValue::Blob(bytes) => Ok(JsonValue::Array(
            bytes.iter().map(|&b| Number::from(b).into()).collect(),
        )),

        // Temporal types
        DuckDBValue::Timestamp(unit, amount) => convert_timestamp(unit, amount),
        DuckDBValue::Date32(days) => convert_date32(days),
        DuckDBValue::Time64(unit, amount) => convert_time64(unit, amount),

        // Complex types
        DuckDBValue::Interval {
            months,
            days,
            nanos,
        } => Ok(JsonValue::Object(Map::from_iter([
            ("months".to_string(), months.into()),
            ("days".to_string(), days.into()),
            ("nanos".to_string(), nanos.into()),
        ]))),

        DuckDBValue::List(list) | DuckDBValue::Array(list) => convert_list(&list),
        DuckDBValue::Struct(items) => convert_struct(&items),
        DuckDBValue::Union(value) => duckdb_value_to_json_value(*value),
        DuckDBValue::Map(items) => convert_map(&items),

        // Special numeric types
        DuckDBValue::HugeInt(i) => Ok(JsonValue::String(i.to_string())),
        DuckDBValue::Decimal(i) => Ok(JsonValue::String(i.to_string())),
    }
}

// Helper functions
fn float_to_json(f: f64) -> JsonValue {
    Number::from_f64(f)
        .map(JsonValue::Number)
        .unwrap_or(JsonValue::Null)
}

fn convert_timestamp(unit: TimeUnit, amount: i64) -> Result<JsonValue> {
    let dt = match unit {
        TimeUnit::Second => DateTime::from_timestamp(amount, 0),
        TimeUnit::Millisecond => DateTime::from_timestamp_millis(amount),
        TimeUnit::Microsecond => DateTime::from_timestamp_micros(amount),
        TimeUnit::Nanosecond => {
            return Ok(JsonValue::String(
                DateTime::from_timestamp_nanos(amount)
                    .format("%+")
                    .to_string(),
            ))
        }
    }
    .context(DuckDBValueConversionSnafu {
        message: "Failed to convert timestamp".to_string(),
    })?;

    Ok(JsonValue::String(dt.format("%+").to_string()))
}

fn convert_date32(days: i32) -> Result<JsonValue> {
    let date = NaiveDate::from_num_days_from_ce_opt(days + 719163).context(
        DuckDBValueConversionSnafu {
            message: "Failed to convert Date32".to_string(),
        },
    )?;

    Ok(JsonValue::String(date.to_string()))
}

fn convert_time64(unit: TimeUnit, amount: i64) -> Result<JsonValue> {
    let micros = unit.to_micros(amount);
    let seconds = micros / 1_000_000;
    let nanos = (micros % 1_000_000) * 1_000;

    let time = NaiveTime::from_num_seconds_from_midnight_opt(
        seconds
            .try_into()
            .map_err(|e| Error::DuckDBValueConversion {
                message: format!("Failed to convert seconds: {}", e),
            })?,
        nanos.try_into().map_err(|e| Error::DuckDBValueConversion {
            message: format!("Failed to convert nanoseconds: {}", e),
        })?,
    )
    .context(DuckDBValueConversionSnafu {
        message: "Failed to create NaiveTime".to_string(),
    })?;

    Ok(JsonValue::String(time.to_string()))
}

fn convert_list(list: &[DuckDBValue]) -> Result<JsonValue> {
    list.iter()
        .map(|item| duckdb_value_to_json_value(item.clone()))
        .collect::<Result<Vec<_>>>()
        .map(JsonValue::Array)
}

fn convert_struct(
    items: &OrderedMap<std::string::String, duckdb::types::Value>,
) -> Result<JsonValue> {
    let mut map = Map::new();
    for (key, value) in items.iter() {
        map.insert(key.clone(), duckdb_value_to_json_value(value.clone())?);
    }
    Ok(JsonValue::Object(map))
}

fn convert_map(items: &OrderedMap<DuckDBValue, DuckDBValue>) -> Result<JsonValue> {
    let mut map = Map::new();
    for (key, value) in items.iter() {
        let key_string = match duckdb_value_to_json_value(key.clone())? {
            JsonValue::String(s) => s,
            JsonValue::Bool(b) => b.to_string(),
            JsonValue::Number(n) => n.to_string(),
            JsonValue::Null => "null".to_string(),
            _ => {
                return DuckDBValueConversionSnafu {
                    message: "Map key must be convertible to string".to_string(),
                }
                .fail()
            }
        };
        map.insert(key_string, duckdb_value_to_json_value(value.clone())?);
    }
    Ok(JsonValue::Object(map))
}

pub fn duckdb_row_to_json(row: &duckdb::Row) -> Result<Vec<JsonValue>> {
    let column_count = row.as_ref().column_count();
    let mut vec = Vec::with_capacity(column_count);

    for i in 0..column_count {
        let value: duckdb::types::Value = row.get(i).map_err(|e| Error::DuckDBValueConversion {
            message: format!("Failed to get value from row {}", e),
        })?;
        // Convert each column value to JSON, wrapping conversion errors as DuckDB errors
        // Using Null as fallback type for conversion errors
        let json_value = duckdb_value_to_json_value(value).map_or(JsonValue::Null, |e| e);
        vec.push(json_value);
    }

    Ok(vec)
}

pub fn sanitize_to_sql_name(filename: &str) -> String {
    const MAX_LENGTH: usize = 63; // Common SQL identifier length limit

    // Sanitize the filename:
    // 1. Replace non-alphanumeric chars with underscore
    // 2. Remove consecutive underscores
    // 3. Remove leading/trailing underscores
    let sanitized: String = filename
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect::<String>()
        .split('_')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("_");

    // If the sanitized string starts with a number, prepend 'n'
    let valid_start = if sanitized
        .chars()
        .next()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(false)
    {
        format!("n{}", sanitized)
    } else {
        sanitized
    };

    // Truncate if necessary, ensuring we don't cut in the middle of an underscore
    if valid_start.len() > MAX_LENGTH {
        let truncated = &valid_start[..MAX_LENGTH];
        match truncated.rfind('_') {
            Some(pos) if pos > 0 => valid_start[..pos].to_string(),
            _ => truncated.to_string(),
        }
    } else {
        valid_start
    }
}
