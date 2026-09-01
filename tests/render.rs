// Feature 6-8: fmt_date, fmt_time, html_escape, render_page.
use alamo::{fmt_date, fmt_time, html_escape, render_page, Film, FilmSchedule};
use chrono::{TimeZone, Utc};

#[test]
fn formats_dates_without_leading_zero() {
    assert_eq!(fmt_date("2026-08-31"), "Mon Aug 31");
    assert_eq!(fmt_date("2026-09-05"), "Sat Sep 5");
    assert_eq!(fmt_date("2026-12-17"), "Thu Dec 17");
}

#[test]
fn formats_times_as_12_hour() {
    assert_eq!(fmt_time("2026-09-05T16:00:00"), "4:00 PM");
    assert_eq!(fmt_time("2026-09-05T09:15:00"), "9:15 AM");
    assert_eq!(fmt_time("2026-09-05T00:00:00"), "12:00 AM");
    assert_eq!(fmt_time("2026-09-05T12:00:00"), "12:00 PM");
}

#[test]
fn escapes_html_special_chars() {
    assert_eq!(html_escape("a & b"), "a &amp; b");
    assert_eq!(html_escape("<script>"), "&lt;script&gt;");
    assert_eq!(html_escape("say \"hi\""), "say &quot;hi&quot;");
    assert_eq!(html_escape("A&B <c> \"d\""), "A&amp;B &lt;c&gt; &quot;d&quot;");
}

fn sample_film(title: &str) -> FilmSchedule {
    sample_film_kind(title, false)
}

fn sample_film_kind(title: &str, is_event: bool) -> FilmSchedule {
    FilmSchedule {
        film: Film {
            slug: "ernie-emma".to_string(),
            title: title.to_string(),
            hero_uri: "https://img-assets.drafthouse.com/images/shows/ernie-emma/HERO.jpg".to_string(),
            is_event,
        },
        dates: vec![("2026-09-05".to_string(), vec!["2026-09-05T16:00:00".to_string()])],
        earliest: Utc.with_ymd_and_hms(2026, 9, 5, 23, 0, 0).unwrap(),
    }
}

#[test]
fn renders_full_document_with_film_content() {
    let html = render_page(&[sample_film("Ernie & Emma")]);
    assert!(html.to_lowercase().contains("<!doctype html>"));
    assert!(html.contains("<meta charset"));
    assert!(html.contains("name=\"viewport\""));
    assert!(html.contains("<style"));
    assert!(html.contains("Alamo Drafthouse DTLA"));
    assert!(html.contains("https://img-assets.drafthouse.com/images/shows/ernie-emma/HERO.jpg"));
    assert!(html.contains("Sat Sep 5"), "formatted date should appear");
    assert!(html.contains("4:00 PM"), "formatted time should appear");
}

#[test]
fn renders_two_column_structure() {
    let html = render_page(&[sample_film("Ernie & Emma")]);

    // Both new wrappers exist.
    assert!(html.contains("<div class=\"film-main\">"), "right-column wrapper present");
    assert!(html.contains("<div class=\"times\">"), "day's time-column wrapper present");

    // Card is hero (left) then film-main (right, holding the showtimes).
    let hero = html.find("class=\"hero\"").expect("hero present");
    let main = html.find("class=\"film-main\"").expect("film-main present");
    let showtimes = html.find("class=\"showtimes\"").expect("showtimes present");
    assert!(hero < main, "hero comes before the right column");
    assert!(main < showtimes, "showtimes live inside film-main");

    // Day row is date (left) then a .times column containing the .time span.
    let date = html.find("class=\"date\"").expect("date present");
    let times = html.find("class=\"times\"").expect("times column present");
    let time = html.find("class=\"time\"").expect("time span present");
    assert!(date < times, "date is the left column");
    assert!(times < time, "the time span sits inside the .times column");
}

#[test]
fn links_hero_and_title() {
    // Regular show: hero anchor + title anchor, both to the /show/ URL.
    let html = render_page(&[sample_film("Ernie & Emma")]);
    let show_url = "https://drafthouse.com/los-angeles/show/ernie-emma?cinemaId=1701";
    assert!(
        html.contains(&format!(
            "<a class=\"hero-link\" href=\"{show_url}\" target=\"_blank\" rel=\"noopener\">"
        )),
        "hero is wrapped in a new-tab link to the show URL"
    );
    assert!(
        html.contains(&format!("<h2><a href=\"{show_url}\" target=\"_blank\" rel=\"noopener\">")),
        "title is wrapped in a new-tab link to the show URL"
    );

    // Event presentations link under /event/ instead.
    let ev = render_page(&[sample_film_kind("Live Q&A", true)]);
    assert!(
        ev.contains("href=\"https://drafthouse.com/event/ernie-emma?cinemaId=1701\""),
        "event card links to the /event/ URL"
    );
}

#[test]
fn render_escapes_titles_to_prevent_injection() {
    let html = render_page(&[sample_film("Ernie & <script>alert(1)</script>")]);
    assert!(html.contains("Ernie &amp; &lt;script&gt;"), "title must be escaped");
    assert!(!html.contains("<script>alert(1)"), "raw script tag must not appear");
}
