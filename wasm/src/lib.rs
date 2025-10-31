use wasm_bindgen::prelude::*;
use serde::Serialize;

#[wasm_bindgen]
extern {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[derive(Serialize)]
pub struct MarkerResult {
    lon: f64,
    lat: f64,
    processed: String,
}

/// Dummy heavy processing (replace as needed)
#[wasm_bindgen]
pub fn heavy_process(input: &str) -> String {
    input.chars().rev().collect()
}

/// Parse CSV and process text in Rust.
/// Input: entire CSV as string. Returns: JSON array.
#[wasm_bindgen]
pub fn parse_and_process_csv(csv_str: &str) -> String {
    let mut results = Vec::new();
    for (i, line) in csv_str.lines().enumerate() {
        let segments: Vec<&str> = line.trim().split(',').collect();
        if segments.len() < 3 { continue; }
        let lon = segments[0].trim().parse::<f64>();
        let lat = segments[1].trim().parse::<f64>();
        let info = &segments[2..].join(",");
        match (lon, lat) {
            (Ok(lon), Ok(lat)) => {
                results.push(MarkerResult {
                    lon,
                    lat,
                    processed: heavy_process(info)
                });
            }
            _ => {
                log(&format!("Skipping line {} (parse error): {}", i, line));
            }
        }
    }
    serde_json::to_string(&results).unwrap_or_else(|_| "[]".into())
}

// Simple echo for tests and demo
#[wasm_bindgen]
pub fn echo(input: &str) -> String {
    input.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_echo() {
        assert_eq!(echo("123"), "123");
    }
}
