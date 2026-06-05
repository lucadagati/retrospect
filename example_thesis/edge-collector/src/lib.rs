use anyhow::Result;
use spin_sdk::http::{IntoResponse, Method, Request, Response};
use spin_sdk::http_component;
use spin_sdk::key_value::Store;

/// Key in the shared Spin KV store where telemetry entries are kept.
const KV_KEY: &str = "telemetry";

/// Edge-collector component — route /edge/...
///
/// Simulates an edge node pushing telemetry to the shared key-value store.
/// Both this component and go-telemetry (/go/...) access the same "default" store.
///
/// POST /edge/add   body=<reading>  → appends "[edge] <reading>" to KV entries
/// GET  /edge/list                  → returns all telemetry entries
#[http_component]
fn handle(req: Request) -> Result<impl IntoResponse> {
    let store = Store::open_default()?;

    match req.method() {
        Method::Post => {
            let text = String::from_utf8_lossy(req.body()).trim().to_string();
            let new_entry = format!("[edge] {}", text);

            // Read existing entries (newline-delimited), append new one, write back.
            let mut entries = load_entries(&store);
            entries.push(new_entry);
            store.set(KV_KEY, entries.join("\n").as_bytes())?;

            Ok(Response::new(200, "edge reading recorded\n"))
        }

        Method::Get => {
            let entries = load_entries(&store);
            let mut out = String::from("=== Telemetry (edge-collector view) ===\n");
            for (i, e) in entries.iter().enumerate() {
                out.push_str(&format!("[{}] {}\n", i, e));
            }
            out.push_str(&format!("total: {} entries\n", entries.len()));
            Ok(Response::new(200, out))
        }

        _ => Ok(Response::new(405, "method not allowed\n")),
    }
}

fn load_entries(store: &Store) -> Vec<String> {
    match store.get(KV_KEY) {
        Ok(Some(bytes)) => String::from_utf8_lossy(&bytes)
            .split('\n')
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect(),
        _ => Vec::new(),
    }
}
