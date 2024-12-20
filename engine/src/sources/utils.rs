use crate::error::{InvalidFormatSnafu, Result};
use std::collections::HashMap;

pub fn source_reader(path: &str, format: &str, params: HashMap<String, String>) -> Result<String> {
    match format {
        "csv" | "tsv" | "txt" => Ok(generate_read_csv_statement(path, params)),
        "parquet" => Ok(generate_read_parquet_statement(path, params)),
        "json" => Ok(generate_read_json_statement(path, params)),
        _ => InvalidFormatSnafu {
            format: format.to_string(),
        }
        .fail(),
    }
}

fn generate_read_csv_statement(path: &str, params: HashMap<String, String>) -> String {
    if params.is_empty() {
        return format!("read_csv('{}')", path);
    }
    let query_params = params
        .iter()
        .map(|(k, v)| format!("{} = '{}'", k, v))
        .collect::<Vec<String>>()
        .join(", ");
    return format!("read_csv('{}', {})", path, query_params);
}

fn generate_read_parquet_statement(path: &str, params: HashMap<String, String>) -> String {
    let query_params = params
        .iter()
        .map(|(k, v)| format!("{} = '{}'", k, v))
        .collect::<Vec<String>>()
        .join(", ");
    return format!("read_parquet('{}', {})", path, query_params);
}

fn generate_read_json_statement(path: &str, params: HashMap<String, String>) -> String {
    let query_params = params
        .iter()
        .map(|(k, v)| format!("{} = '{}'", k, v))
        .collect::<Vec<String>>()
        .join(", ");
    return format!("read_json('{}', {})", path, query_params);
}
