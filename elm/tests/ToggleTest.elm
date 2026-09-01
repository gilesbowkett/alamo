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
    init sampleDays


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
        , test "closed: label 'show dates/times', no showtimes" <|
            \_ ->
                view start
                    |> Query.fromHtml
                    |> Expect.all
                        [ Query.has [ text "show dates/times" ]
                        , Query.hasNot [ class "showtimes" ]
                        ]
        , test "open: label 'hide dates/times' and the showtime link" <|
            \_ ->
                view opened
                    |> Query.fromHtml
                    |> Expect.all
                        [ Query.has [ text "hide dates/times" ]
                        , Query.has [ class "showtimes" ]
                        , Query.has [ class "time", text "4:00 PM" ]
                        ]
        ]
