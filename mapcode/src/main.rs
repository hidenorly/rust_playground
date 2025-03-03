/*
  Copyright (C) 2025 hidenorly

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
*/

use reqwest::blocking::Client;
use std::env;
use std::process;

fn get_mapcode(latitude: f64, longitude: f64) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new();
    let lat_str = latitude.to_string();
    let lon_str = longitude.to_string();

    let params = [
        ("t", "jpndeg"),
        ("jpn_lat", lat_str.as_str()),
        ("jpn_lon", lon_str.as_str()),
    ];
    let response = client.post("https://saibara.sakura.ne.jp/map/convgeo.cgi")
        .form(&params)
        .send()?;
    let body = response.text()?;
    let start_tag = r#"name="mapcode" value=""#;
    if let Some(start) = body.find(start_tag) {
        let start_index = start + start_tag.len();
        if let Some(end_index) = body[start_index..].find('"') {
            let mapcode = &body[start_index..start_index + end_index];
            return Ok(mapcode.to_string());
        }
    }
    
    Err("Failed to extract mapcode from response".into())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: cargo run <latitude> <longitude>");
        process::exit(1);
    }

    let latitude: f64 = args[1].parse().expect("Invalid latitude");
    let longitude: f64 = args[2].parse().expect("Invalid longitude");

    match get_mapcode(latitude, longitude) {
        Ok(mapcode) => println!("Mapcode: {}", mapcode),
        Err(e) => eprintln!("Error: {}", e),
    }
}
