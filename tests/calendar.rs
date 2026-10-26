// build_calendar: film-bearing weeks grouped under month labels (no fixed window).
use alamo::{build_calendar, Film, FilmSchedule};
use chrono::{TimeZone, Utc};

fn film(slug: &str, title: &str, dates: &[&str]) -> FilmSchedule {
    FilmSchedule {
        film: Film {
            slug: slug.to_string(),
            title: title.to_string(),
            hero_uri: "https://img/x.jpg".to_string(),
            is_event: false,
            runtime_minutes: None,
        },
        dates: dates
            .iter()
            .map(|d| (d.to_string(), vec![format!("{d}T16:00:00")]))
            .collect(),
        earliest: Utc.with_ymd_and_hms(2026, 9, 5, 23, 0, 0).unwrap(),
    }
}

#[test]
fn no_films_means_no_weeks() {
    let now = Utc.with_ymd_and_hms(2026, 9, 2, 12, 0, 0).unwrap();
    assert!(build_calendar(&[], now).is_empty());
}

#[test]
fn shows_only_film_bearing_weeks_each_labelled_with_its_month() {
    let now = Utc.with_ymd_and_hms(2026, 9, 2, 12, 0, 0).unwrap();
    // One September film and one December film, nothing between.
    let sept = film("ernie-emma", "Ernie & Emma", &["2026-09-07"]); // week 9/6–9/12
    let dec = film("avengers", "Avengers", &["2026-12-17"]); // week 12/13–12/19
    let weeks = build_calendar(&[sept, dec], now);

    // Exactly the two film-bearing weeks — no current week, no barren Oct/Nov.
    assert_eq!(weeks.len(), 2, "only the two weeks that have a film");
    let months: Vec<&str> = weeks.iter().map(|w| w.month.as_str()).collect();
    assert_eq!(months, vec!["September", "December"]);

    // Each shown week still carries all seven of its days.
    assert!(weeks.iter().all(|w| w.days.len() == 7));
}

#[test]
fn straddling_week_belongs_to_the_month_that_begins_in_it() {
    let now = Utc.with_ymd_and_hms(2026, 9, 2, 12, 0, 0).unwrap();
    // Week Sun 11/29–Sat 12/5 straddles: December begins (Tue 12/1) within it.
    let straddle = film("straddle", "Straddle", &["2026-11-30"]);
    let weeks = build_calendar(&[straddle], now);
    assert_eq!(weeks.len(), 1);
    assert_eq!(weeks[0].month, "December", "a month beginning in the week claims it");

    // A week wholly inside November stays November.
    let whole = film("whole", "Whole", &["2026-11-15"]); // week 11/15–11/21
    let weeks = build_calendar(&[whole], now);
    assert_eq!(weeks[0].month, "November");
}

#[test]
fn a_film_appears_on_its_day_within_a_full_week() {
    let now = Utc.with_ymd_and_hms(2026, 9, 2, 12, 0, 0).unwrap();
    let f = film("ernie-emma", "Ernie & Emma", &["2026-09-07"]); // id 0, Monday
    let weeks = build_calendar(&[f], now);

    assert_eq!(weeks.len(), 1);
    let days = &weeks[0].days;
    assert_eq!(days.len(), 7);
    // The week runs Sun 9/6 .. Sat 9/12; the film sits only on 9/7.
    for d in days {
        let expected = d.date == "9/7";
        assert_eq!(d.film_ids.contains(&0), expected, "film placement wrong on {}", d.date);
    }
}
