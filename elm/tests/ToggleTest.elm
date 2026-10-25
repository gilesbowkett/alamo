module ToggleTest exposing (suite)

import Expect
import Test exposing (Test, describe, test)
import Test.Html.Query as Query
import Test.Html.Selector exposing (class, text)
import Toggle exposing (Msg(..), init, update, view)


sampleDays : List Toggle.Day
sampleDays =
    [ Toggle.Day "Sat Sep 5"
        [ Toggle.Slot "4:00 PM" "https://drafthouse.com/los-angeles/show/x?cinemaId=1701&date=2026-09-05" ]
    ]


start : Toggle.Model
start =
    init "1 hour 30 minutes" sampleDays


opened : Toggle.Model
opened =
    update Toggle start


suite : Test
suite =
    describe "Toggle component"
        [ test "starts closed" <|
            \_ -> Expect.equal False start.open
        , test "Toggle flips open" <|
            \_ -> Expect.equal True opened.open
        , test "closed: label 'showtimes', no showtimes" <|
            \_ ->
                view start
                    |> Query.fromHtml
                    |> Expect.all
                        [ Query.has [ text "showtimes" ]
                        , Query.hasNot [ class "showtimes" ]
                        ]
        , test "open: label 'showtimes', runtime above the day rows, and the showtime link" <|
            \_ ->
                view opened
                    |> Query.fromHtml
                    |> Expect.all
                        [ Query.has [ text "showtimes" ]
                        , Query.has [ class "showtimes" ]
                        , Query.has [ class "runtime", text "1 hour 30 minutes" ]
                        , Query.has [ class "time", text "4:00 PM" ]
                        ]
        , test "open with no runtime: no runtime div" <|
            \_ ->
                update Toggle (init "" sampleDays)
                    |> view
                    |> Query.fromHtml
                    |> Query.hasNot [ class "runtime" ]
        ]
