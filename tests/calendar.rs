// build_calendar: the 4-week grid and its film-on-day membership.
use alamo::{build_calendar, Film, FilmSchedule};
use chrono::{TimeZone, Utc};

fn film(slug: &str, title: &str, dates: &[&str]) -> FilmSchedule {
    FilmSchedule {
        film: Film {
            slug: slug.to_string(),
            title: title.to_string(),
            hero_uri: "https://img/x.jpg".to_string(),
            is_event: false,
        },
        dates: dates
            .iter()
            .map(|d| (d.to_string(), vec![format!("{d}T16:00:00")]))
            .collect(),
        earliest: Utc.with_ymd_and_hms(2026, 9, 5, 23, 0, 0).unwrap(),
    }
}

#[test]
fn grid_is_four_weeks_from_the_current_sunday_with_ghosting() {
    let now = Utc.with_ymd_and_hms(2026, 9, 2, 12, 0, 0).unwrap(); // Wednesday
    let cal = build_calendar(&[], now);

    assert_eq!(cal.len(), 28, "7 columns x 4 rows");
    // Week starts Sunday 2026-08-30; M/D has no leading zeros or year.
    assert_eq!(cal[0].date, "8/30");
    assert_eq!(cal[3].date, "9/2"); // Sun=0 .. Wed=3 == today
    // Days before today are ghosted; today and later are not.
    assert!(cal[0].ghosted);
    assert!(cal[2].ghosted, "the day before today is ghosted");
    assert!(!cal[3].ghosted, "today is active");
    assert!(!cal[4].ghosted, "future is active");
}

#[test]
fn a_film_appears_on_exactly_the_days_it_plays() {
    let now = Utc.with_ymd_and_hms(2026, 9, 2, 12, 0, 0).unwrap();
    let in_window = film("ernie-emma", "Ernie & Emma", &["2026-09-05", "2026-09-06"]);
    let out_of_window = film("avengers", "Avengers", &["2026-12-17"]);
    let cal = build_calendar(&[in_window, out_of_window], now);

    // Ernie plays 9/5 and 9/6 — and nowhere else.
    let ernie_days: Vec<&str> = cal
        .iter()
        .filter(|d| d.films.iter().any(|t| t == "Ernie & Emma"))
        .map(|d| d.date.as_str())
        .collect();
    assert_eq!(ernie_days, vec!["9/5", "9/6"]);

    // Every day that isn't 9/5 or 9/6 lacks Ernie (no false positives).
    for d in &cal {
        let expected = d.date == "9/5" || d.date == "9/6";
        assert_eq!(
            d.films.iter().any(|t| t == "Ernie & Emma"),
            expected,
            "Ernie presence wrong on {}",
            d.date
        );
    }

    // A film playing only outside the 4-week window appears on no day.
    assert!(cal.iter().all(|d| d.films.iter().all(|t| t != "Avengers")));
}
