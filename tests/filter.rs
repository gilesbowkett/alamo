// Feature 3: is_dtla_future — keep only DTLA (cinema 1701) sessions in the future.
use alamo::{is_dtla_future, parse_sessions, Session};
use chrono::{TimeZone, Utc};
use std::fs;

fn session(cinema: &str, utc: &str) -> Session {
    Session {
        presentation_slug: "ernie-emma".to_string(),
        cinema_id: cinema.to_string(),
        show_time_clt: "2026-09-05T16:00:00".to_string(),
        show_time_utc: utc.to_string(),
        business_date_clt: "2026-09-05".to_string(),
    }
}

#[test]
fn dtla_future_session_is_kept() {
    let now = Utc.with_ymd_and_hms(2026, 9, 1, 0, 0, 0).unwrap();
    assert!(is_dtla_future(&session("1701", "2026-09-05T23:00:00"), now));
}

#[test]
fn non_dtla_session_is_dropped() {
    let now = Utc.with_ymd_and_hms(2026, 9, 1, 0, 0, 0).unwrap();
    assert!(!is_dtla_future(&session("9999", "2026-09-05T23:00:00"), now));
}

#[test]
fn past_dtla_session_is_dropped() {
    // now is AFTER the showtime
    let now = Utc.with_ymd_and_hms(2026, 9, 6, 0, 0, 0).unwrap();
    assert!(!is_dtla_future(&session("1701", "2026-09-05T23:00:00"), now));
}

#[test]
fn mixed_cinemas_fixture_drops_the_flipped_session() {
    // mixed_cinemas.json is ernie-emma with one session flipped to cinema 9999.
    let json = fs::read_to_string(format!(
        "{}/tests/fixtures/mixed_cinemas.json",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    let sessions = parse_sessions(&json).unwrap();
    let now = Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 0).unwrap(); // before every showtime
    let kept = sessions.iter().filter(|s| is_dtla_future(s, now)).count();
    assert_eq!(kept, 9, "one non-1701 session should be filtered out, leaving 9");
}
