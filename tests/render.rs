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

fn now() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 9, 1, 0, 0, 0).unwrap()
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
    let html = render_page(&[sample_film("Ernie & Emma")], now());
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
fn emits_container_mount_and_runtime() {
    let html = render_page(&[sample_film("Ernie & Emma")], now());

    // The whole film list is rendered by the container Elm app: one mount + the
    // inlined runtime + an init call. No server-rendered cards.
    assert!(html.contains("<div id=\"app\" data-flags=\""), "container mount present");
    assert!(html.contains("Elm.Main.init"), "container is booted");
    assert!(html.contains("Elm.Main"), "elm runtime is inlined");
    assert!(!html.contains("<article"), "cards are not server-rendered");
    assert!(!html.contains("<div class=\"showtimes\">"), "showtimes are not server-rendered");
}

#[test]
fn container_flags_carry_each_films_data() {
    // A /show/ film's flags carry id, title, hero, header url, RT url, and its
    // showtimes (day label + time label + show-date href).
    let html = render_page(&[sample_film("Ernie & Emma")], now());
    for needle in [
        "&quot;id&quot;:&quot;ernie-emma&quot;", // JSON quotes are HTML-escaped in the attribute
        "&quot;title&quot;:&quot;Ernie &amp; Emma&quot;",
        "img-assets.drafthouse.com/images/shows/ernie-emma/HERO.jpg",
        "los-angeles/show/ernie-emma?cinemaId=1701",
        "rottentomatoes.com/search?search=ernie-emma",
        "Sat Sep 5",
        "4:00 PM",
        "date=2026-09-05",
        "&quot;calendar&quot;:",              // the calendar grid rides along in flags
        "&quot;date&quot;:&quot;9/5&quot;",   // the film's showday has a cell
    ] {
        assert!(html.contains(needle), "flags should contain {needle:?}, got:\n{html}");
    }

    // Events use the /event/ base in their header/showtime URLs.
    let ev = render_page(&[sample_film_kind("Live Q&A", true)], now());
    assert!(
        ev.contains("event/ernie-emma?cinemaId=1701"),
        "event film links under /event/, got:\n{ev}"
    );
}

#[test]
fn render_escapes_titles_to_prevent_injection() {
    let html = render_page(&[sample_film("Ernie & <script>alert(1)</script>")], now());
    assert!(html.contains("Ernie &amp; &lt;script&gt;"), "title must be escaped");
    assert!(!html.contains("<script>alert(1)"), "raw script tag must not appear");
}
