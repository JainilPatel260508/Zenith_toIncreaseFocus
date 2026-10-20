// Development-only stdin/stdout bridge. Never part of the app or DMG.
use serde::Deserialize;
use std::io::{self, BufRead, Write};
use zenith_lib::{Database, Request};
#[derive(Deserialize)]
struct Input {
    id: u64,
    request: Request,
}
fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("Provide an isolated verification database path");
    let mut db =
        Database::open(std::path::Path::new(&path)).expect("Could not open verification database");
    for line in io::stdin().lock().lines() {
        let line = line.expect("Read stdin");
        let response = match serde_json::from_str::<Input>(&line) {
            Ok(input) => match zenith_lib::handle_request(&mut db, input.request) {
                Ok(result) => serde_json::json!({"id":input.id,"result":result}),
                Err(error) => serde_json::json!({"id":input.id,"error":error}),
            },
            Err(error) => serde_json::json!({"error":error.to_string()}),
        };
        println!("{response}");
        io::stdout().flush().expect("Flush result");
    }
}
