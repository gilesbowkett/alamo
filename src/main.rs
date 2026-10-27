// Thin I/O shell: fetch the DTLA market schedule feed, then fetch each displayed
// film's per-presentation feed for its runtime (the market feed omits it), hand
// the data to the pure `alamo` pipeline, and print the HTML page to stdout. Any
// fetch/parse error that survives retries aborts (exit 1). All testable logic
// lives in src/lib.rs.

use std::error::Error;
use std::process;
use std::time::Duration;

use chrono::Utc;
use rayon::prelude::*;
use reqwest::blocking::Client;

// One feed lists every presentation in the Los Angeles market plus all of their
// sessions. The API returns 403 without a browser-like User-Agent.
const MARKET_URL: &str = "https://drafthouse.com/s/mother/v2/schedule/market/los-angeles";
// The per-presentation feed, keyed by presentation slug. Unlike the market feed,
// it populates `runtimeMinutes`.
const PRESENTATION_BASE: &str = "https://drafthouse.com/s/mother/v2/schedule/presentation/los-angeles";
const USER_AGENT: &str =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 \
     (KHTML, like Gecko) Chrome/124.0 Safari/537.36";

// Bounded concurrency for the per-film runtime fetches: this work is I/O-bound,
// so we allow more in-flight requests than CPUs without going unbounded.
const FETCH_THREADS: usize = 12;
// Per-request retry policy: perfect results or stop. A failed request is retried
// with exponential backoff up to this many attempts, then aborts the run.
const MAX_ATTEMPTS: u32 = 5;
const BACKOFF_BASE_MS: u64 = 200;

/// GET `url` as text, retrying transient failures with exponential backoff. After
/// `MAX_ATTEMPTS` the final error is returned (which aborts the run — we never
/// render a partial page). `String` error so it can cross rayon worker threads.
fn fetch_text(client: &Client, url: &str) -> Result<String, String> {
    let mut attempt = 0;
    loop {
        attempt += 1;
        let result = client
            .get(url)
            .send()
            .and_then(|r| r.error_for_status())
            .and_then(|r| r.text());
        match result {
            Ok(body) => return Ok(body),
            Err(e) => {
                if attempt >= MAX_ATTEMPTS {
                    return Err(format!(
                        "GET {url} failed after {attempt} attempts: {}",
                        alamo::error_chain(&e)
                    ));
                }
                let backoff = BACKOFF_BASE_MS * 2u64.pow(attempt - 1);
                std::thread::sleep(Duration::from_millis(backoff));
            }
        }
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let client = Client::builder().user_agent(USER_AGENT).build()?;
    let now = Utc::now();

    let market_json = fetch_text(&client, MARKET_URL)?;

    // The films we will display (future DTLA, soonest-first). We fetch a runtime
    // for each — the market feed left them null.
    let mut schedules = alamo::schedule(&market_json, now)?;

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(FETCH_THREADS)
        .build()?;
    pool.install(|| -> Result<(), String> {
        schedules.par_iter_mut().try_for_each(|s| {
            let url = format!("{PRESENTATION_BASE}/{}", s.film.slug);
            let body = fetch_text(&client, &url)?;
            s.film.runtime_minutes = alamo::parse_runtime(&body, &s.film.slug).map_err(|e| {
                format!("parsing runtime for {}: {}", s.film.slug, alamo::error_chain(&e))
            })?;
            Ok(())
        })
    })?;

    print!("{}", alamo::render_page(&schedules, now));
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("alamo: error: {e}");
        process::exit(1);
    }
}
