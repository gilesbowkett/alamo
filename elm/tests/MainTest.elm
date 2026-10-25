module MainTest exposing (suite)

import Expect
import Json.Encode as E
import Main exposing (Msg(..), Tab(..), init, update, view)
import Showtimes
import Test exposing (Test, describe, test)
import Test.Html.Query as Query
import Test.Html.Selector exposing (class, tag, text)


slot : String -> E.Value
slot label =
    E.object [ ( "label", E.string label ), ( "href", E.string "https://example.com/x?cinemaId=1701&date=2026-09-05" ) ]


day : String -> String -> String -> E.Value
day key date label =
    E.object
        [ ( "key", E.string key )
        , ( "date", E.string date )
        , ( "times", E.list identity [ slot label ] )
        ]


film : Int -> String -> String -> E.Value
film id title timeLabel =
    let
        s =
            String.fromInt id
    in
    E.object
        [ ( "id", E.int id )
        , ( "title", E.string title )
        , ( "hero", E.string ("https://img/" ++ s ++ ".jpg") )
        , ( "url", E.string ("https://drafthouse.com/los-angeles/show/" ++ s ++ "?cinemaId=1701") )
        , ( "rt", E.string ("https://www.rottentomatoes.com/search?search=" ++ s) )
        , ( "runtime", E.string "1 hour 30 minutes" )
        , ( "days", E.list identity [ day "2026-09-05" "Sat Sep 5" timeLabel ] )
        ]


calDay : String -> String -> Bool -> List Int -> E.Value
calDay key date ghosted filmIds =
    E.object
        [ ( "key", E.string key )
        , ( "date", E.string date )
        , ( "ghosted", E.bool ghosted )
        , ( "filmIds", E.list E.int filmIds )
        ]


flags : E.Value
flags =
    E.object
        [ ( "films", E.list identity [ film 0 "Alpha" "4:00 PM", film 1 "Beta" "9:00 PM" ] )

        -- Both films play 9/5; the calendar resolves them via the films dict.
        , ( "calendar", E.list identity [ calDay "2026-08-30" "8/30" True [], calDay "2026-09-05" "9/5" False [ 0, 1 ] ] )
        ]


start : Main.Model
start =
    Tuple.first (init flags)


suite : Test
suite =
    describe "Container app"
        [ test "renders both films with a remove control each" <|
            \_ ->
                view start
                    |> Query.fromHtml
                    |> Expect.all
                        [ Query.has [ text "Alpha" ]
                        , Query.has [ text "Beta" ]
                        , Query.findAll [ class "remove" ] >> Query.count (Expect.equal 2)
                        , Query.has [ text "\u{2717}" ]
                        ]
        , test "HideFilm drops that film from the list, keeps the other" <|
            \_ ->
                Tuple.first (update (HideFilm 0) start)
                    |> view
                    |> Query.fromHtml
                    |> Expect.all
                        [ Query.hasNot [ text "Alpha" ]
                        , Query.has [ text "Beta" ]
                        ]
        , test "HideFilm also removes the film from the calendar, keeps the other" <|
            \_ ->
                -- The bug an earlier refactor fixed: hiding in list mode must hide
                -- in calendar mode too, since both resolve through model.films.
                start
                    |> update (HideFilm 0)
                    |> Tuple.first
                    |> update (SetTab CalendarTab)
                    |> Tuple.first
                    |> view
                    |> Query.fromHtml
                    |> Expect.all
                        [ Query.has [ class "calendar" ]
                        , Query.hasNot [ text "Alpha" ]
                        , Query.has [ class "calendar-film", text "Beta" ]
                        ]
        , test "ShowtimesMsg opens only that film's showtimes" <|
            \_ ->
                Tuple.first (update (ShowtimesMsg 0 Showtimes.Toggle) start)
                    |> view
                    |> Query.fromHtml
                    |> Expect.all
                        [ Query.has [ class "time", text "4:00 PM" ]
                        , Query.hasNot [ text "9:00 PM" ]
                        ]
        , test "tabs show both labels, list active by default, no calendar" <|
            \_ ->
                view start
                    |> Query.fromHtml
                    |> Expect.all
                        [ Query.has [ text "list" ]
                        , Query.has [ text "calendar" ]
                        , Query.has [ class "film" ]
                        , Query.hasNot [ class "calendar" ]
                        ]
        , test "CalendarTab shows each film as title + hero + a showtimes link, times hidden by default" <|
            \_ ->
                Tuple.first (update (SetTab CalendarTab) start)
                    |> view
                    |> Query.fromHtml
                    |> Expect.all
                        [ Query.has [ class "calendar" ]
                        , Query.has [ class "calendar-day", text "9/5" ]
                        , Query.has [ class "calendar-film", text "Alpha" ]
                        , Query.findAll [ class "calendar-hero", tag "img" ] >> Query.count (Expect.equal 2)
                        , Query.findAll [ class "toggle", text "showtimes" ] >> Query.count (Expect.equal 2)
                        , Query.hasNot [ class "time" ]
                        , Query.hasNot [ class "film" ]
                        ]
        , test "ToggleCalendarFilm opens only that film's showtimes on that day (times + runtime)" <|
            \_ ->
                start
                    |> update (SetTab CalendarTab)
                    |> Tuple.first
                    |> update (ToggleCalendarFilm 0 "2026-09-05")
                    |> Tuple.first
                    |> view
                    |> Query.fromHtml
                    |> Expect.all
                        [ Query.has [ class "time", text "4:00 PM" ]
                        , Query.has [ class "runtime" ]
                        -- film 1 on the same day stays closed
                        , Query.hasNot [ class "time", text "9:00 PM" ]
                        ]
        ]
