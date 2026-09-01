// Feature 1-2: deserialize the market feed — presentations and sessions.
use std::fs;

fn fixture(name: &str) -> String {
    fs::read_to_string(format!("{}/tests/fixtures/{}", env!("CARGO_MANIFEST_DIR"), name))
        .unwrap_or_else(|e| panic!("read fixture {name}: {e}"))
}

#[test]
fn parses_presentations_from_market() {
    let films = alamo::parse_presentations(&fixture("market.json")).expect("parse presentations");
    assert_eq!(films.len(), 64, "market feed currently lists 64 presentations");

    let ernie = films
        .iter()
        .find(|f| f.slug == "ernie-emma")
        .expect("ernie-emma present in market feed");
    assert_eq!(ernie.title, "Ernie & Emma");
    assert!(
        ernie.hero_uri.starts_with("https://img-assets.drafthouse.com/images/shows/ernie-emma/"),
        "hero_uri should be the landscape hero image, got: {}",
        ernie.hero_uri
    );

    // Film.slug must be the presentation-level slug, not show.slug: this special
    // event shares show `ernie-emma` but has its own presentation slug.
    assert!(
        films.iter().any(|f| f.slug == "live-q-a-ernie-emma"),
        "presentation-level slug live-q-a-ernie-emma should be its own film"
    );
}

#[test]
fn builds_show_and_event_urls() {
    let films = alamo::parse_presentations(&fixture("market.json")).expect("parse presentations");
    let find = |slug: &str| films.iter().find(|f| f.slug == slug).unwrap();

    let show = find("tony");
    assert!(!show.is_event, "a regular presentation is not an event");
    assert_eq!(
        alamo::presentation_url(show),
        "https://drafthouse.com/los-angeles/show/tony?cinemaId=1701"
    );

    let event = find("the-twilight-saga-twilight-2008-fan-event");
    assert!(event.is_event, "a presentation with an `event` object is an event");
    assert_eq!(
        alamo::presentation_url(event),
        "https://drafthouse.com/event/the-twilight-saga-twilight-2008-fan-event?cinemaId=1701"
    );
}

#[test]
fn parses_sessions_from_market() {
    // A single-presentation capture: 10 sessions across two presentation slugs.
    let sessions = alamo::parse_sessions(&fixture("ernie-emma.json")).expect("parse sessions");
    assert_eq!(sessions.len(), 10, "ernie-emma fixture has 10 sessions");

    let s = &sessions[0];
    assert_eq!(s.cinema_id, "1701");
    assert_eq!(s.show_time_clt, "2026-09-05T16:00:00");
    assert_eq!(s.show_time_utc, "2026-09-05T23:00:00");
    assert_eq!(s.business_date_clt, "2026-09-05");
    assert_eq!(s.presentation_slug, "ernie-emma", "session carries its presentation slug");
}
