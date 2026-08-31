// Feature 1-2: deserialize the two feed shapes from real captured fixtures.
use std::fs;

fn fixture(name: &str) -> String {
    fs::read_to_string(format!("{}/tests/fixtures/{}", env!("CARGO_MANIFEST_DIR"), name))
        .unwrap_or_else(|e| panic!("read fixture {name}: {e}"))
}

#[test]
fn parses_featured_into_films() {
    let films = alamo::parse_featured(&fixture("featured.json")).expect("parse featured");
    assert_eq!(films.len(), 8, "featured feed currently lists 8 films");

    let ernie = films
        .iter()
        .find(|f| f.slug == "ernie-emma")
        .expect("ernie-emma present in featured");
    assert_eq!(ernie.title, "Ernie & Emma");
    assert!(
        ernie.hero_uri.starts_with("https://img-assets.drafthouse.com/images/shows/ernie-emma/"),
        "hero_uri should be the landscape hero image, got: {}",
        ernie.hero_uri
    );
}

#[test]
fn parses_sessions_from_presentation_feed() {
    let sessions = alamo::parse_sessions(&fixture("ernie-emma.json")).expect("parse sessions");
    assert_eq!(sessions.len(), 10, "ernie-emma has 10 sessions");

    let s = &sessions[0];
    assert_eq!(s.cinema_id, "1701");
    assert_eq!(s.show_time_clt, "2026-09-05T16:00:00");
    assert_eq!(s.show_time_utc, "2026-09-05T23:00:00");
    assert_eq!(s.business_date_clt, "2026-09-05");
}
