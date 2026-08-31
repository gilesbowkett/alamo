// Thin I/O shell: fetch the two feed types, hand the JSON to the pure `alamo`
// pipeline, print the HTML page to stdout. Any fetch/parse error aborts (exit 1).
// All testable logic lives in src/lib.rs.

use std::error::Error;
use std::process;

use chrono::Utc;

const MARKET: &str = "los-angeles";
const FEATURED_URL: &str = "https://drafthouse.com/s/mother/v2/schedule/featured/los-angeles";
// The API returns 403 without a browser-like User-Agent.
const USER_AGENT: &str =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 \
     (KHTML, like Gecko) Chrome/124.0 Safari/537.36";

fn presentation_url(slug: &str) -> String {
    format!("https://drafthouse.com/s/mother/v2/schedule/presentation/{MARKET}/{slug}")
}

fn run() -> Result<(), Box<dyn Error>> {
    let client = reqwest::blocking::Client::builder()
        .user_agent(USER_AGENT)
        .build()?;

    let get = |url: &str| -> Result<String, Box<dyn Error>> {
        let body = client
            .get(url)
            .send()?
            .error_for_status()? // 4xx/5xx -> abort
            .text()?;
        Ok(body)
    };

    // Featured feed -> the list of films to look up.
    let featured_json = get(FEATURED_URL)?;
    let films = alamo::parse_featured(&featured_json)?;

    // One presentation feed per film.
    let mut film_jsons: Vec<(String, String)> = Vec::with_capacity(films.len());
    for film in &films {
        let json = get(&presentation_url(&film.slug))?;
        film_jsons.push((film.slug.clone(), json));
    }

    let film_refs: Vec<(&str, &str)> = film_jsons
        .iter()
        .map(|(slug, json)| (slug.as_str(), json.as_str()))
        .collect();

    let html = alamo::build_page(&featured_json, &film_refs, Utc::now())?;
    print!("{html}");
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("alamo: error: {e}");
        process::exit(1);
    }
}
