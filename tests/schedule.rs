// Feature 4-5: group_by_date, and building/sorting per-film schedules.
use alamo::{group_by_date, parse_featured, parse_sessions, FilmSchedule, Session};
use chrono::{TimeZone, Utc};
use std::fs;

fn fixture(name: &str) -> String {
    fs::read_to_string(format!("{}/tests/fixtures/{}", env!("CARGO_MANIFEST_DIR"), name)).unwrap()
}

fn session(clt: &str, date: &str) -> Session {
    Session {
        cinema_id: "1701".to_string(),
        show_time_clt: clt.to_string(),
        show_time_utc: format!("{clt}Z-ignored"), // unused by group_by_date
        business_date_clt: date.to_string(),
    }
}

#[test]
fn groups_by_date_ascending_with_times_sorted() {
    // Deliberately unordered input across two days.
    let sessions = vec![
        session("2026-09-06T19:30:00", "2026-09-06"),
        session("2026-09-05T16:00:00", "2026-09-05"),
        session("2026-09-05T09:15:00", "2026-09-05"),
        session("2026-09-06T12:00:00", "2026-09-06"),
    ];
    let grouped = group_by_date(&sessions);
    assert_eq!(
        grouped,
        vec![
            (
                "2026-09-05".to_string(),
                vec!["2026-09-05T09:15:00".to_string(), "2026-09-05T16:00:00".to_string()]
            ),
            (
                "2026-09-06".to_string(),
                vec!["2026-09-06T12:00:00".to_string(), "2026-09-06T19:30:00".to_string()]
            ),
        ]
    );
}

fn schedule_from(fixture_name: &str, film_slug: &str, film_title: &str) -> FilmSchedule {
    let films = parse_featured(&fixture("featured.json")).unwrap();
    let film = films.into_iter().find(|f| f.slug == film_slug).unwrap();
    assert_eq!(film.title, film_title);
    let sessions = parse_sessions(&fixture(fixture_name)).unwrap();
    // "now" before all sessions in either fixture so nothing is filtered as past.
    let now = Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 0).unwrap();
    FilmSchedule::build(film, &sessions, now).expect("film has future DTLA sessions")
}

#[test]
fn sorts_films_by_earliest_showtime_avengers_last() {
    let ernie = schedule_from("ernie-emma.json", "ernie-emma", "Ernie & Emma"); // September
    let avengers =
        schedule_from("avengers-doomsday.json", "avengers-doomsday", "Avengers: Doomsday"); // December

    // Insert in the "wrong" order to prove sorting does the work.
    let mut films = vec![avengers, ernie];
    alamo::sort_films(&mut films);

    assert_eq!(films[0].film.slug, "ernie-emma", "September film sorts first");
    assert_eq!(films[1].film.slug, "avengers-doomsday", "December film sorts last");

    // ernie-emma spans 7 DTLA dates (2026-09-04 .. 2026-09-10).
    let ernie_dates: Vec<&str> = films[0].dates.iter().map(|(d, _)| d.as_str()).collect();
    assert_eq!(
        ernie_dates,
        vec![
            "2026-09-04", "2026-09-05", "2026-09-06", "2026-09-07", "2026-09-08", "2026-09-09",
            "2026-09-10"
        ]
    );
}
