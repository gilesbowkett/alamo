// Thin I/O shell: fetch the DTLA market schedule feed, hand the JSON to the pure
// `alamo` pipeline, print the HTML page to stdout. Any fetch/parse error aborts
// (exit 1). All testable logic lives in src/lib.rs.

use std::error::Error;
use std::process;

use chrono::Utc;

// One feed lists every presentation in the Los Angeles market plus all of their
// sessions. The API returns 403 without a browser-like User-Agent.
const MARKET_URL: &str = "https://drafthouse.com/s/mother/v2/schedule/market/los-angeles";
const USER_AGENT: &str =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 \
     (KHTML, like Gecko) Chrome/124.0 Safari/537.36";

fn run() -> Result<(), Box<dyn Error>> {
    let client = reqwest::blocking::Client::builder()
        .user_agent(USER_AGENT)
        .build()?;

    let response = client
        .get(MARKET_URL)
        .send()
        .and_then(|r| r.error_for_status()) // 4xx/5xx -> abort
        .map_err(|e| format!("GET {MARKET_URL} failed: {}", alamo::error_chain(&e)))?;

    let market_json = response.text().map_err(|e| {
        format!(
            "GET {MARKET_URL} succeeded but decoding the response body failed: {}",
            alamo::error_chain(&e)
        )
    })?;

    let html = alamo::build_page(&market_json, Utc::now())?;
    print!("{html}");
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("alamo: error: {e}");
        process::exit(1);
    }
}
