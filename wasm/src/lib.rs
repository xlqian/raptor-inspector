use polars::prelude::*;
use serde::Serialize;
use serde_wasm_bindgen;
use std::{collections::HashSet, io::Cursor};
use wasm_bindgen::prelude::*;
mod movix;
use movix::*;

#[wasm_bindgen]
extern "C" {
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
        if segments.len() < 3 {
            continue;
        }
        let lon = segments[0].trim().parse::<f64>();
        let lat = segments[1].trim().parse::<f64>();
        let info = &segments[2..].join(",");
        match (lon, lat) {
            (Ok(lon), Ok(lat)) => {
                results.push(MarkerResult {
                    lon,
                    lat,
                    processed: heavy_process(info),
                });
            }
            _ => {
                log(&format!("Skipping line {} (parse error): {}", i, line));
            }
        }
    }
    serde_json::to_string(&results).unwrap_or_else(|_| "[]".into())
}

#[wasm_bindgen]
pub fn parse_and_process_raptor_output(input: &str) -> movix::raptor_output::RaptorOutput {
    movix::raptor_output::RaptorOutput::new(input)
}

#[wasm_bindgen]
pub struct MyDataFrame {
    df: DataFrame,
}

#[wasm_bindgen]
impl MyDataFrame {
    #[wasm_bindgen(constructor)]
    pub fn new(input: &str) -> MyDataFrame {
        let cursor = Cursor::new(input);
        let df = CsvReadOptions::default()
            .with_parse_options(CsvParseOptions::default().with_separator(b';'))
            .with_has_header(true)
            .into_reader_with_file_handle(cursor)
            .finish()
            .unwrap();
        MyDataFrame { df }
    }

    #[wasm_bindgen]
    pub fn row_count(&self) -> usize {
        self.df.height()
    }
}

impl MyDataFrame {
    pub fn coords_of_stop(&self, stop_offset: i64) -> Option<(f64, f64)> {
        let filter = self
            .df
            .column("StopOffset")
            .unwrap()
            .i64()
            .unwrap()
            .equal(stop_offset as i64);
        let filtered_df = self.df.filter(&filter).unwrap();

        let lon_series = filtered_df.column("StopLng").unwrap().f64().unwrap();
        let lat_series = filtered_df.column("StopLat").unwrap().f64().unwrap();

        if lon_series.len() == 0 || lat_series.len() == 0 {
            return None;
        }

        let lon = lon_series.get(0).unwrap();
        let lat = lat_series.get(0).unwrap();

        Some((lon, lat))
    }

    pub fn details_of_stops(&self, stop_offsets: &Vec<i64>) -> Vec<Option<(f64, f64, String)>> {
        let id_set: HashSet<&i64> = stop_offsets.iter().collect();

        let filter = self
            .df
            .column("StopOffset")
            .unwrap()
            .i64()
            .unwrap()
            .into_iter()
            .map(|v| v.map_or(false, |id| id_set.contains(&id)))
            .collect::<BooleanChunked>();

        let matching_rows = self.df.filter(&filter).unwrap();
        let lon_series = matching_rows.column("StopLng").unwrap().f64().unwrap();
        let lat_series = matching_rows.column("StopLat").unwrap().f64().unwrap();
        let name_series = matching_rows.column("Stopname").unwrap().str().unwrap();

        let mut coords = Vec::new();
        for ((lon, lat), name) in lon_series
            .iter()
            .zip(lat_series.iter())
            .zip(name_series.iter())
        {
            coords.push(Some((
                lon.unwrap(),
                lat.unwrap(),
                name.unwrap().to_string(),
            )));
        }
        coords
    }
}

// Simple echo for tests and demo
#[wasm_bindgen]
pub fn echo(input: &str) -> String {
    input.to_string()
}

#[wasm_bindgen]
pub fn stops_details_of_round(
    data: &MyDataFrame,
    raptor_output: &movix::raptor_output::RaptorOutput,
    round_index: usize,
) -> JsValue {
    let coords: Vec<(f64, f64)> = Vec::new();
    if round_index >= raptor_output.rounds_number() {
        return serde_wasm_bindgen::to_value(&coords).unwrap();
    }

    let stop_offsets = raptor_output.stop_offsets_of_round(round_index);
    let coords = data
        .details_of_stops(&stop_offsets)
        .into_iter()
        .flatten()
        .collect::<Vec<(f64, f64, String)>>();

    serde_wasm_bindgen::to_value(&coords).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_echo() {
        assert_eq!(echo("123"), "123");
    }
    #[test]
    fn test_parse_and_process_raptor_output() {
        let input = r#"round,0,
1,2,3,
round,1,
route,43,
route,42,1,2,3,4,5,6,7,8,9,
round,2,
marked_stop,42,
marked_stop,43,45,89,78,
"#;
        let raptor_output = parse_and_process_raptor_output(input);
        assert_eq!(raptor_output.rounds_number(), 3);
    }
}
