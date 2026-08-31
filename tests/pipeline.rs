// Feature 9: build_page — the pure pipeline over real fixtures.
use alamo::build_page;
use chrono::{TimeZone, Utc};
use std::fs;

fn fixture(name: &str) -> String {
    fs::read_to_string(format!("{}/tests/fixtures/{}", env!("CARGO_MANIFEST_DIR"), name)).unwrap()
}

#[test]
fn orders_films_and_keeps_correct_dates() {
    let featured = fixture("featured.json");
    let films = [
        ("avengers-doomsday", fixture("avengers-doomsday.json")),
        ("ernie-emma", fixture("ernie-emma.json")),
    ];
    let film_refs: Vec<(&str, &str)> = films.iter().map(|(s, j)| (*s, j.as_str())).collect();
    let now = Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 0).unwrap();

    let html = build_page(&featured, &film_refs, now).unwrap();

    // September film must render before the December film.
    let ernie_at = html.find("Ernie &amp; Emma").expect("ernie present");
    let avengers_at = html.find("Avengers: Doomsday").expect("avengers present");
    assert!(ernie_at < avengers_at, "September film sorts before December film");

    // Correct date labels present for each.
    assert!(html.contains("Sat Sep 5"), "ernie September date");
    assert!(html.contains("Thu Dec 17"), "avengers December date");
    // ernie spans 7 days -> its earliest label is Fri Sep 4.
    assert!(html.contains("Fri Sep 4"));
}

#[test]
fn drops_non_dtla_sessions() {
    // mixed_cinemas.json = ernie-emma with one session flipped to cinema 9999.
    let featured = fixture("featured.json");
    let films = [("ernie-emma", fixture("mixed_cinemas.json"))];
    let film_refs: Vec<(&str, &str)> = films.iter().map(|(s, j)| (*s, j.as_str())).collect();
    let now = Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 0).unwrap();

    let html = build_page(&featured, &film_refs, now).unwrap();
    let times = html.matches("class=\"time\"").count();
    assert_eq!(times, 9, "10 sessions minus the one non-DTLA session = 9 showtimes");
}

#[test]
fn far_future_now_drops_all_past_sessions() {
    let featured = fixture("featured.json");
    let films = [
        ("ernie-emma", fixture("ernie-emma.json")),
        ("avengers-doomsday", fixture("avengers-doomsday.json")),
    ];
    let film_refs: Vec<(&str, &str)> = films.iter().map(|(s, j)| (*s, j.as_str())).collect();
    let now = Utc.with_ymd_and_hms(2027, 1, 1, 0, 0, 0).unwrap(); // after everything

    let html = build_page(&featured, &film_refs, now).unwrap();
    assert!(!html.contains("<article"), "no films should render when all sessions are past");
}
