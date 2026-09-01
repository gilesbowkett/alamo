// alamo: pure logic for turning Alamo Drafthouse schedule JSON into an HTML page.
// All network I/O lives in main.rs; everything here is pure and unit-tested.

use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};
use std::error::Error;

/// Format an error and its `source()` chain as "msg: cause: deeper-cause", so a
/// terse top-level message (e.g. reqwest's "error decoding response body") is
/// reported alongside its real underlying cause.
pub fn error_chain(err: &dyn Error) -> String {
    let mut out = err.to_string();
    let mut src = err.source();
    while let Some(e) = src {
        out.push_str(": ");
        out.push_str(&e.to_string());
        src = e.source();
    }
    out
}

pub const DTLA_CINEMA_ID: &str = "1701";

/// A film (presentation) from the market feed: what we need to render its card
/// header. `slug` is the presentation-level slug — the key sessions join on.
#[derive(Debug, Clone)]
pub struct Film {
    pub slug: String,
    pub title: String,
    pub hero_uri: String,
    /// True when this presentation is an `/event/` page rather than a `/show/`.
    pub is_event: bool,
}

/// One showtime session from the market feed. `presentation_slug` joins it to
/// the `Film` whose showtime it is.
#[derive(Debug, Clone)]
pub struct Session {
    pub presentation_slug: String,
    pub cinema_id: String,
    pub show_time_clt: String,
    pub show_time_utc: String,
    pub business_date_clt: String,
}

// --- Deserialization shapes: only the fields we consume; serde ignores the rest. ---

#[derive(Deserialize)]
struct ScheduleDoc {
    data: ScheduleData,
}
#[derive(Deserialize)]
struct ScheduleData {
    presentations: Vec<SchedulePresentation>,
    sessions: Vec<RawSession>,
}
#[derive(Deserialize)]
struct SchedulePresentation {
    slug: String,
    show: RawShow,
    // Present (non-null) only for event pages; its contents are irrelevant here.
    #[serde(default)]
    event: Option<serde::de::IgnoredAny>,
}
#[derive(Deserialize)]
struct RawShow {
    title: String,
    #[serde(rename = "landscapeHeroImage")]
    landscape_hero_image: RawImage,
}
#[derive(Deserialize)]
struct RawImage {
    uri: String,
}

#[derive(Deserialize)]
struct RawSession {
    #[serde(rename = "presentationSlug")]
    presentation_slug: String,
    #[serde(rename = "cinemaId")]
    cinema_id: String,
    #[serde(rename = "showTimeClt")]
    show_time_clt: String,
    #[serde(rename = "showTimeUtc")]
    show_time_utc: String,
    #[serde(rename = "businessDateClt")]
    business_date_clt: String,
}

/// Format a business date "2026-09-05" as "Sat Sep 5". Falls back to the raw string.
pub fn fmt_date(date: &str) -> String {
    match NaiveDate::parse_from_str(date, "%Y-%m-%d") {
        Ok(d) => d.format("%a %b %-d").to_string(),
        Err(_) => date.to_string(),
    }
}

/// Format a CLT timestamp "2026-09-05T16:00:00" as "4:00 PM". Falls back to the raw string.
pub fn fmt_time(clt: &str) -> String {
    match parse_naive(clt) {
        Some(dt) => dt.format("%-I:%M %p").to_string(),
        None => clt.to_string(),
    }
}

/// Escape text for safe interpolation into HTML element content / double-quoted attrs.
pub fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}

/// Render the full HTML page for the given films (already filtered and sorted).
pub fn render_page(films: &[FilmSchedule]) -> String {
    let mut h = String::new();
    h.push_str("<!doctype html>\n<html lang=\"en\">\n<head>\n");
    h.push_str("<meta charset=\"utf-8\">\n");
    h.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
    h.push_str("<title>Alamo Drafthouse DTLA — Upcoming Showtimes</title>\n");
    h.push_str("<style>\n");
    h.push_str(STYLE);
    h.push_str("</style>\n</head>\n<body>\n");
    h.push_str("<h1>Alamo Drafthouse DTLA — Upcoming Showtimes</h1>\n");

    for film in films {
        let title = html_escape(&film.film.title);
        let hero = html_escape(&film.film.hero_uri);
        let url = html_escape(&presentation_url(&film.film));
        h.push_str("<article class=\"film\">\n");
        h.push_str(&format!(
            "  <a class=\"hero-link\" href=\"{url}\" target=\"_blank\" rel=\"noopener\">\
             <img class=\"hero\" src=\"{hero}\" alt=\"{title}\" loading=\"lazy\"></a>\n"
        ));
        h.push_str("  <div class=\"film-main\">\n");
        h.push_str(&format!(
            "    <h2><a href=\"{url}\" target=\"_blank\" rel=\"noopener\">{title}</a></h2>\n"
        ));
        h.push_str("    <div class=\"showtimes\">\n");
        for (date, times) in &film.dates {
            let day = html_escape(&fmt_date(date));
            let slots: Vec<String> = times
                .iter()
                .map(|t| format!("<span class=\"time\">{}</span>", html_escape(&fmt_time(t))))
                .collect();
            h.push_str(&format!(
                "      <div class=\"day\"><strong class=\"date\">{day}</strong>\
                 <div class=\"times\">{}</div></div>\n",
                slots.join(" ")
            ));
        }
        h.push_str("    </div>\n  </div>\n</article>\n");
    }

    h.push_str("</body>\n</html>\n");
    h
}

const STYLE: &str = r#"
  :root { color-scheme: light; }
  body { font-family: system-ui, -apple-system, Segoe UI, Roboto, sans-serif;
         margin: 0 auto; padding: 1.5rem 1rem; line-height: 1.4;
         color: #1a1a1a; background: #fff; }
  h1 { font-size: 1.6rem; margin: 0 0 1.5rem; }
  .film { display: grid; grid-template-columns: 1fr 1fr; gap: 1rem 1.25rem;
          align-items: start; margin: 0 0 2.5rem; }
  .film-main { min-width: 0; }
  .hero-link { display: block; }
  .hero { display: block; width: 100%; max-width: 100%; height: auto;
          border-radius: 8px; background: #eee; }
  .film h2 { font-size: 1.25rem; margin: 0 0 0.5rem; }
  .film h2 a { color: inherit; text-decoration: none; }
  .showtimes { display: flex; flex-direction: column; gap: 0.4rem; }
  .day { display: grid; grid-template-columns: 6.5rem 1fr; align-items: baseline;
         gap: 0.4rem 0.6rem; }
  .date { color: #444; }
  .times { display: flex; flex-wrap: wrap; gap: 0.4rem; }
  .time { display: inline-block; padding: 0.15rem 0.5rem; border: 1px solid #ccc;
          border-radius: 4px; font-variant-numeric: tabular-nums; white-space: nowrap; }
"#;

/// Parse an Alamo CLT/UTC timestamp string (e.g. "2026-09-05T23:00:00", no zone suffix).
fn parse_naive(ts: &str) -> Option<NaiveDateTime> {
    NaiveDateTime::parse_from_str(ts, "%Y-%m-%dT%H:%M:%S").ok()
}

/// True if the session is at DTLA and its UTC showtime is strictly after `now`.
/// A session whose UTC timestamp can't be parsed is treated as not-future (dropped).
pub fn is_dtla_future(session: &Session, now: DateTime<Utc>) -> bool {
    if session.cinema_id != DTLA_CINEMA_ID {
        return false;
    }
    match parse_naive(&session.show_time_utc) {
        Some(naive) => naive.and_utc() > now,
        None => false,
    }
}

/// The pure pipeline: the market feed JSON + the current time → the finished HTML
/// page. Sessions are joined to their presentation by `presentation_slug`; films
/// with no future DTLA session are dropped; the rest are ordered soonest-first.
/// Propagates a JSON parse error rather than emitting a partial page.
pub fn build_page(market_json: &str, now: DateTime<Utc>) -> Result<String, serde_json::Error> {
    let films = parse_presentations(market_json)?;
    let sessions = parse_sessions(market_json)?;

    let mut by_slug: HashMap<String, Vec<Session>> = HashMap::new();
    for s in sessions {
        by_slug.entry(s.presentation_slug.clone()).or_default().push(s);
    }

    let mut schedules = Vec::new();
    for film in films {
        let film_sessions = by_slug.get(&film.slug).map(Vec::as_slice).unwrap_or(&[]);
        if let Some(sched) = FilmSchedule::build(film, film_sessions, now) {
            schedules.push(sched);
        }
    }
    sort_films(&mut schedules);
    Ok(render_page(&schedules))
}

/// Group sessions by their business date (ascending), with each day's showtimes
/// (the raw CLT timestamp strings) sorted ascending.
pub fn group_by_date(sessions: &[Session]) -> Vec<(String, Vec<String>)> {
    let mut by_date: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for s in sessions {
        by_date
            .entry(s.business_date_clt.clone())
            .or_default()
            .push(s.show_time_clt.clone());
    }
    by_date
        .into_iter()
        .map(|(date, mut times)| {
            times.sort();
            (date, times)
        })
        .collect()
}

/// A film paired with its DTLA showtimes, grouped by date and ready to render.
#[derive(Debug, Clone)]
pub struct FilmSchedule {
    pub film: Film,
    /// (business date, sorted CLT showtime strings), dates ascending.
    pub dates: Vec<(String, Vec<String>)>,
    /// Earliest future DTLA showtime, used to order films.
    pub earliest: DateTime<Utc>,
}

impl FilmSchedule {
    /// Build a schedule from a film and its raw sessions, keeping only future DTLA
    /// sessions. Returns `None` if the film has no such sessions (so it is dropped).
    pub fn build(film: Film, sessions: &[Session], now: DateTime<Utc>) -> Option<FilmSchedule> {
        let future: Vec<Session> = sessions
            .iter()
            .filter(|s| is_dtla_future(s, now))
            .cloned()
            .collect();
        let earliest = future
            .iter()
            .filter_map(|s| parse_naive(&s.show_time_utc).map(|n| n.and_utc()))
            .min()?;
        Some(FilmSchedule {
            film,
            dates: group_by_date(&future),
            earliest,
        })
    }
}

/// Sort films by their earliest upcoming showtime, soonest first.
pub fn sort_films(films: &mut [FilmSchedule]) {
    films.sort_by_key(|f| f.earliest);
}

/// Parse the market feed JSON into the presentations (films) it lists.
pub fn parse_presentations(json: &str) -> Result<Vec<Film>, serde_json::Error> {
    let doc: ScheduleDoc = serde_json::from_str(json)?;
    Ok(doc
        .data
        .presentations
        .into_iter()
        .map(|p| Film {
            slug: p.slug,
            title: p.show.title,
            hero_uri: p.show.landscape_hero_image.uri,
            is_event: p.event.is_some(),
        })
        .collect())
}

/// The public Drafthouse URL for a film, scoped to the DTLA cinema. Event pages
/// live under `/event/`; regular shows under `/los-angeles/show/`.
pub fn presentation_url(film: &Film) -> String {
    if film.is_event {
        format!("https://drafthouse.com/event/{}?cinemaId={DTLA_CINEMA_ID}", film.slug)
    } else {
        format!(
            "https://drafthouse.com/los-angeles/show/{}?cinemaId={DTLA_CINEMA_ID}",
            film.slug
        )
    }
}

/// Parse the market feed JSON into all of its sessions.
pub fn parse_sessions(json: &str) -> Result<Vec<Session>, serde_json::Error> {
    let doc: ScheduleDoc = serde_json::from_str(json)?;
    Ok(doc
        .data
        .sessions
        .into_iter()
        .map(|s| Session {
            presentation_slug: s.presentation_slug,
            cinema_id: s.cinema_id,
            show_time_clt: s.show_time_clt,
            show_time_utc: s.show_time_utc,
            business_date_clt: s.business_date_clt,
        })
        .collect())
}
