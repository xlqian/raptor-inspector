use polars::prelude::*;
use serde::Serialize;
use serde_wasm_bindgen;
use std::{collections::HashSet, io::Cursor};
use wasm_bindgen::prelude::*;

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

#[derive(Debug)]
struct InitRound {
    marked_stops: Vec<i64>,
}

#[derive(Debug)]
struct Route {
    route_id: i64,
    explored_stops: Vec<i64>,
}
#[derive(Debug)]
struct RouteRound {
    explored_routes: Vec<Route>,
}

#[derive(Debug)]
struct Transfer {
    marked_stop: i64,
    reached_stops: Vec<i64>,
}

#[derive(Debug)]
struct TransferRound {
    explored_transfers: Vec<Transfer>,
}

#[derive(Debug)]
enum Round {
    Init(InitRound),
    Route(RouteRound),
    Transfer(TransferRound),
    None,
}

// alias type
#[wasm_bindgen]
pub struct RaptorOutput {
    rounds: Vec<Round>,
}

#[wasm_bindgen]
impl RaptorOutput {
    #[wasm_bindgen(constructor)]
    pub fn new(raptor_output_str: &str) -> Self {
        let mut rounds = Vec::new();

        let mut current_round = Round::None;

        for (_, line) in raptor_output_str.lines().enumerate() {
            if line.is_empty() {
                continue;
            }

            if line.starts_with("round") {
                let round_number = line
                    .split(",")
                    .filter_map(|s| s.parse::<u8>().ok())
                    .last()
                    .unwrap();

                if round_number == 0 {
                    current_round = Round::Init(InitRound {
                        marked_stops: Vec::new(),
                    });
                } else {
                    let to_be_moved = std::mem::replace(&mut current_round, Round::None);
                    rounds.push(to_be_moved);
                    println!("{:?}", rounds);

                    match round_number % 2 {
                        1 => {
                            current_round = Round::Route(RouteRound {
                                explored_routes: Vec::new(),
                            });
                        }
                        0 => {
                            current_round = Round::Transfer(TransferRound {
                                explored_transfers: Vec::new(),
                            });
                        }
                        _ => {}
                    }
                }
            } else {
                match current_round {
                    Round::Init(ref mut round) => {
                        // split then parse into int
                        let stop_indexes = line.split(",").filter_map(|s| s.parse::<i64>().ok());
                        round.marked_stops.extend(stop_indexes);
                    }
                    Round::Route(ref mut round) => {
                        let mut parts = line
                            .split(',')
                            .skip(1)
                            .filter_map(|s| s.parse::<i64>().ok());

                        let route_id = parts.next().unwrap();
                        let explored_stops = parts.collect();
                        round.explored_routes.push(Route {
                            route_id,
                            explored_stops,
                        });
                    }
                    Round::Transfer(ref mut round) => {
                        let mut parts = line
                            .split(',')
                            .skip(1)
                            .filter_map(|s| s.parse::<i64>().ok());

                        let marked_stop = parts.next().unwrap();
                        let reached_stops = parts.collect();
                        round.explored_transfers.push(Transfer {
                            marked_stop,
                            reached_stops,
                        });
                    }
                    Round::None => {}
                }
            }
        }
        rounds.push(current_round);

        RaptorOutput { rounds }
    }

    #[wasm_bindgen]
    pub fn rounds_number(&self) -> usize {
        self.rounds.len()
    }

    #[wasm_bindgen]
    pub fn called_by_TS(&self) -> String {
        "TOTO".to_string()
    }
}

#[wasm_bindgen]
pub fn parse_and_process_raptor_output(input: &str) -> RaptorOutput {
    RaptorOutput::new(input)
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
    raptor_output: &RaptorOutput,
    round_index: usize,
) -> JsValue {
    let coords: Vec<(f64, f64)> = Vec::new();
    if round_index >= raptor_output.rounds.len() {
        return serde_wasm_bindgen::to_value(&coords).unwrap();
    }
    let coords = match &raptor_output.rounds[round_index] {
        Round::Init(init_round) => data
            .details_of_stops(&init_round.marked_stops)
            .into_iter()
            .filter_map(|opt| opt)
            .collect::<Vec<(f64, f64, String)>>(),
        Round::Route(route_round) => {
            let ids = route_round
                .explored_routes
                .iter()
                .flat_map(|route| route.explored_stops.iter().cloned())
                .collect::<Vec<i64>>();
            data.details_of_stops(&ids)
                .into_iter()
                .filter_map(|opt| opt)
                .collect::<Vec<(f64, f64, String)>>()
        }
        Round::Transfer(transfer_round) => {
            let ids = transfer_round
                .explored_transfers
                .iter()
                .flat_map(|transfer| transfer.reached_stops.iter().cloned())
                .collect::<Vec<i64>>();

            data.details_of_stops(&ids)
                .into_iter()
                .filter_map(|opt| opt)
                .collect::<Vec<(f64, f64, String)>>()
        }
        Round::None => Vec::new(),
    };
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
        assert_eq!(raptor_output.rounds.len(), 3);
    }
}
