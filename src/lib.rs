// alamo: pure logic for turning Alamo Drafthouse schedule JSON into an HTML page.
// All network I/O lives in main.rs; everything here is pure and unit-tested.

use chrono::{DateTime, Datelike, Duration, NaiveDate, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
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
    /// Runtime in whole minutes. `None` from the market feed (which never sets it);
    /// populated from the per-presentation endpoint. See `parse_runtime`.
    pub runtime_minutes: Option<u32>,
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
    // Null in the market feed; populated by the per-presentation endpoint.
    #[serde(rename = "runtimeMinutes", default)]
    runtime_minutes: Option<u32>,
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

/// Format a runtime in minutes as e.g. "1 hour 54 minutes". `None` or 0 → "" so
/// the UI can omit the runtime entirely. Units are singularized at 1.
pub fn fmt_runtime(minutes: Option<u32>) -> String {
    let m = match minutes {
        Some(m) if m > 0 => m,
        _ => return String::new(),
    };
    let (hours, mins) = (m / 60, m % 60);
    let mut parts = Vec::new();
    if hours > 0 {
        parts.push(format!("{hours} hour{}", if hours == 1 { "" } else { "s" }));
    }
    if mins > 0 {
        parts.push(format!("{mins} minute{}", if mins == 1 { "" } else { "s" }));
    }
    parts.join(" ")
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

// The compiled Elm visibility-toggle app (built by build.rs), inlined so the
// page stays self-contained.
const ELM_JS: &str = include_str!(concat!(env!("OUT_DIR"), "/elm.js"));

// Boots the container app with the whole film list as flags.
const INIT_JS: &str = "<script>\n\
(function () {\n\
  var app = document.getElementById('app');\n\
  Elm.Main.init({ node: app, flags: JSON.parse(app.getAttribute('data-flags')) });\n\
})();\n\
</script>\n";

// --- Elm flags: every film, with already-formatted strings. ---
#[derive(Serialize)]
struct PageFlags {
    films: Vec<FilmFlag>,
    calendar: Vec<CalendarDay>,
}

/// One cell of the 4-week calendar grid: its `M/D` date, whether it is a past
/// (ghosted) day, and the ids of the films playing that day. The ids are foreign
/// keys into the film list (see `film_flag`); the client resolves them to titles,
/// so a hidden film disappears from the calendar too.
#[derive(Serialize)]
pub struct CalendarDay {
    // ISO date ("2026-09-05") — matches each film's FlagDay.key so the calendar
    // can scope a film's Showtimes to this day. `date` is the "M/D" display.
    pub key: String,
    pub date: String,
    pub ghosted: bool,
    #[serde(rename = "filmIds")]
    pub film_ids: Vec<u32>,
}

/// Build the 4-week calendar grid (7 columns × 4 rows) starting at the Sunday of
/// the week containing `now`. Each cell lists the ids of the films playing that
/// day (in soonest-first film order); days before today are ghosted.
pub fn build_calendar(films: &[FilmSchedule], now: DateTime<Utc>) -> Vec<CalendarDay> {
    let today = now.date_naive();
    let offset = today.weekday().num_days_from_sunday() as i64;
    let start = today - Duration::days(offset);

    // business date (YYYY-MM-DD) -> film ids, accumulated in film order. A film's
    // id is its index here, matching `film_flag`'s id and the client's Dict key.
    let mut by_date: HashMap<String, Vec<u32>> = HashMap::new();
    for (i, film) in films.iter().enumerate() {
        for (date, _) in &film.dates {
            by_date.entry(date.clone()).or_default().push(i as u32);
        }
    }

    (0..28)
        .map(|i| {
            let day = start + Duration::days(i);
            let iso = day.format("%Y-%m-%d").to_string();
            CalendarDay {
                date: format!("{}/{}", day.month(), day.day()),
                ghosted: day < today,
                film_ids: by_date.get(&iso).cloned().unwrap_or_default(),
                key: iso,
            }
        })
        .collect()
}
#[derive(Serialize)]
struct FilmFlag {
    // The film's id: its index in the (soonest-first) film list. Doubles as the
    // client's Dict key and the foreign key the calendar cells reference.
    id: u32,
    title: String,
    hero: String,
    url: String,
    rt: String,
    // Pre-formatted runtime, e.g. "1 hour 54 minutes"; "" when unknown.
    runtime: String,
    days: Vec<FlagDay>,
}
#[derive(Serialize)]
struct FlagDay {
    // ISO business date ("2026-09-05") — the stable key a calendar cell scopes a
    // film's Showtimes to. `date` is the human display ("Sat Sep 5").
    key: String,
    date: String,
    times: Vec<FlagSlot>,
}
#[derive(Serialize)]
struct FlagSlot {
    label: String,
    href: String,
}

/// Build one film's flags: its header URL (`presentation_url`), Rotten Tomatoes
/// search, and showtimes (each linking to the header URL with the day appended).
fn film_flag(id: u32, film: &FilmSchedule) -> FilmFlag {
    let base = presentation_url(&film.film);
    let days = film
        .dates
        .iter()
        .map(|(date, times)| FlagDay {
            key: date.clone(),
            date: fmt_date(date),
            times: times
                .iter()
                .map(|t| FlagSlot {
                    label: fmt_time(t),
                    href: format!("{base}&date={date}"),
                })
                .collect(),
        })
        .collect();
    FilmFlag {
        id,
        title: film.film.title.clone(),
        hero: film.film.hero_uri.clone(),
        rt: format!("https://www.rottentomatoes.com/search?search={}", film.film.slug),
        runtime: fmt_runtime(film.film.runtime_minutes),
        url: base,
        days,
    }
}

/// Render the full HTML page for the given films (already filtered and sorted).
/// `now` anchors the calendar grid.
pub fn render_page(films: &[FilmSchedule], now: DateTime<Utc>) -> String {
    let mut h = String::new();
    h.push_str("<!doctype html>\n<html lang=\"en\">\n<head>\n");
    h.push_str("<meta charset=\"utf-8\">\n");
    h.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
    h.push_str("<title>Alamo Drafthouse DTLA: Upcoming Showtimes</title>\n");
    h.push_str("<style>\n");
    h.push_str(STYLE);
    h.push_str("</style>\n");
    h.push_str("<script>");
    h.push_str(ELM_JS);
    h.push_str("</script>\n</head>\n<body>\n");
    h.push_str("<h1>Alamo Drafthouse DTLA: Upcoming Showtimes</h1>\n");

    // The container Elm app renders every card from these flags and owns the
    // per-film remove/toggle interactions.
    let flags = PageFlags {
        films: films.iter().enumerate().map(|(i, f)| film_flag(i as u32, f)).collect(),
        calendar: build_calendar(films, now),
    };
    let flags_json = serde_json::to_string(&flags).expect("flags serialize");
    h.push_str(&format!(
        "<div id=\"app\" data-flags=\"{}\"></div>\n",
        html_escape(&flags_json)
    ));

    h.push_str(INIT_JS);
    h.push_str("</body>\n</html>\n");
    h
}

const STYLE: &str = r#"
  :root { color-scheme: light; --cream: #fdfcf8; --ink: #1b1d24; }
  body { font-family: system-ui, -apple-system, Segoe UI, Roboto, sans-serif;
         margin: 0 auto; padding: 1.5rem 1rem; line-height: 1.4;
         color: var(--ink); background: var(--cream); }
  h1 { font-size: 1.6rem; margin: 0 0 1.5rem; }
  .tabs { display: flex; gap: 0.5rem; margin: 0 0 1.5rem; }
  .tab { display: inline-flex; align-items: center; gap: 0.4rem;
         padding: 0.35rem 0.8rem; border: 1px solid #ccc; border-radius: 6px;
         background: transparent; color: var(--ink); font: inherit;
         font-variant: small-caps; cursor: pointer; }
  .tab .icon { width: 1em; height: 1em; }
  .tab.active { background: var(--ink); color: var(--cream); border-color: var(--ink); }
  .calendar { display: grid; grid-template-columns: repeat(7, 1fr); gap: 0.4rem; }
  .calendar-day { border: 1px solid #ddd; border-radius: 6px; padding: 0.4rem;
             min-height: 5rem; font-size: 0.85rem; }
  .calendar-day.ghost { opacity: 0.45; }
  .calendar-date { font-variant-numeric: tabular-nums; color: #666; margin: 0 0 0.3rem; }
  .calendar-film { margin: 0 0 0.6rem; }
  .calendar-title { font-weight: 600; margin: 0 0 0.2rem; }
  .calendar-hero { display: block; width: 96%; height: auto; border-radius: 4px;
              background: #eee; margin: 0 0 0.25rem; }
  .film { display: grid; grid-template-columns: 1fr 1fr; gap: 1rem 1.25rem;
          align-items: start; margin: 0 0 2.5rem; }
  .film-main { min-width: 0; }
  .hero-wrap { position: relative; }
  .hero-link { display: block; }
  .hero { display: block; width: 100%; max-width: 100%; height: auto;
          border-radius: 8px; background: #eee; }
  .remove { position: absolute; top: 0.4rem; left: 0.4rem;
            width: 1.6rem; height: 1.6rem; padding: 0; border: none;
            border-radius: 50%; background: var(--cream); color: var(--ink);
            font-size: 1rem; line-height: 1; cursor: pointer;
            display: flex; align-items: center; justify-content: center; }
  .remove:hover { background: #efefef; }
  .calendar-film .remove { width: 1.1rem; height: 1.1rem; font-size: 0.7rem;
                           top: 0.25rem; left: 0.25rem; }
  .film h2 { font-size: 1.25rem; margin: 0 0 0.5rem; }
  .film h2 a { color: inherit; text-decoration: none; }
  .rt { font-variant: small-caps; color: #d00; text-decoration: none;
        font-size: 0.8rem; margin-right: 0.75rem; }
  .rt:hover { color: #a80000; }
  .rt:active { color: #000; }
  .toggle { font-variant: small-caps; color: #06c; text-decoration: none;
            font-size: 0.8rem; cursor: pointer; }
  .toggle:hover { color: #004d99; }
  .toggle:active { color: #000; }
  /* Elm's toggle view root vanishes from layout so the toggle sits inline next
     to .rt while the showtimes drop to their own line below. */
  .contents { display: contents; }
  .showtimes { display: flex; flex-direction: column; gap: 0.4rem;
               margin-top: 0.6rem; }
  .runtime { font-variant: small-caps; color: #666; font-size: 0.8rem;
             margin-left: 0.75rem; }
  .calendar-film .runtime { display: block; margin-left: 0; margin-bottom: 6px; }
  .day { display: grid; grid-template-columns: 6.5rem 1fr; align-items: baseline;
         gap: 0.4rem 0.6rem; }
  .date { color: #444; }
  .times { display: flex; flex-wrap: wrap; gap: 0.4rem; }
  .time { display: inline-block; padding: 0.15rem 0.5rem; border: 1px solid #ccc;
          border-radius: 4px; font-variant-numeric: tabular-nums; white-space: nowrap;
          color: inherit; text-decoration: none; }
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
    Ok(render_page(&schedule(market_json, now)?, now))
}

/// The parse/join/filter/sort half of the pipeline: the market feed JSON → the
/// films to display (future DTLA only, soonest-first), without rendering. The I/O
/// shell uses this to learn which films to display so it can fetch each one's
/// runtime (absent from the market feed) before handing the lot to `render_page`.
pub fn schedule(market_json: &str, now: DateTime<Utc>) -> Result<Vec<FilmSchedule>, serde_json::Error> {
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
    Ok(schedules)
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
            runtime_minutes: p.show.runtime_minutes,
        })
        .collect())
}

/// Extract a presentation's runtime (minutes) from a per-presentation feed
/// response, keyed by its presentation slug. `Ok(None)` when the feed carries no
/// runtime for that slug (absent presentation or null runtime) — that is the true
/// value, not a failure. Propagates a JSON parse error.
pub fn parse_runtime(json: &str, slug: &str) -> Result<Option<u32>, serde_json::Error> {
    let films = parse_presentations(json)?;
    Ok(films.into_iter().find(|f| f.slug == slug).and_then(|f| f.runtime_minutes))
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
