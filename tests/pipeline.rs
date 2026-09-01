// Feature 9: build_page — the pure pipeline over the whole market feed.
use alamo::build_page;
use chrono::{TimeZone, Utc};
use std::fs;

fn fixture(name: &str) -> String {
    fs::read_to_string(format!("{}/tests/fixtures/{}", env!("CARGO_MANIFEST_DIR"), name)).unwrap()
}

#[test]
fn renders_every_film_in_the_market_feed() {
    let market = fixture("market.json");
    let now = Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 0).unwrap(); // before all sessions

    let html = build_page(&market, now).unwrap();

    // One entry per presentation with future DTLA sessions in the container's
    // flags — far more than the old featured feed's 8.
    assert!(html.contains("<div id=\"app\" data-flags=\""), "container mount present");
    let films = html.matches("&quot;id&quot;:&quot;").count();
    assert_eq!(films, 64, "every market presentation appears in the flags");

    // Titles that were NOT in the featured feed must now appear.
    assert!(html.contains("Practical Magic 2"), "non-featured film present");
    assert!(html.contains("It Ends"), "non-featured film present");
}

#[test]
fn orders_films_soonest_first() {
    let market = fixture("market.json");
    let now = Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 0).unwrap();

    let html = build_page(&market, now).unwrap();

    // Spider-Man's earliest DTLA showtime (Sep 1) precedes Avengers' (Dec 17).
    let spidey = html.find("Spider-Man: Brand New Day").expect("spider-man present");
    let avengers = html.find("Avengers: Doomsday").expect("avengers present");
    assert!(spidey < avengers, "September film sorts before December film");
}

#[test]
fn far_future_now_drops_all_past_sessions() {
    let market = fixture("market.json");
    let now = Utc.with_ymd_and_hms(2028, 1, 1, 0, 0, 0).unwrap(); // after everything

    let html = build_page(&market, now).unwrap();
    assert!(html.contains("&quot;films&quot;:[]"), "flags carry no films when all sessions are past");
}
