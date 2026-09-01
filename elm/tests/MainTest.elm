module MainTest exposing (suite)

import Expect
import Json.Encode as E
import Main exposing (Msg(..), init, update, view)
import Test exposing (Test, describe, test)
import Test.Html.Query as Query
import Test.Html.Selector exposing (class, text)


sampleFlags : E.Value
sampleFlags =
    E.object
        [ ( "days"
          , E.list identity
                [ E.object
                    [ ( "date", E.string "Sat Sep 5" )
                    , ( "times"
                      , E.list identity
                            [ E.object
                                [ ( "label", E.string "4:00 PM" )
                                , ( "href", E.string "https://drafthouse.com/los-angeles/show/x?cinemaId=1701&date=2026-09-05" )
                                ]
                            ]
                      )
                    ]
                ]
          )
        ]


start : Main.Model
start =
    Tuple.first (init sampleFlags)


opened : Main.Model
opened =
    Tuple.first (update Toggle start)


suite : Test
suite =
    describe "Main visibility toggle"
        [ test "starts closed" <|
            \_ -> Expect.equal False start.open
        , test "Toggle flips open" <|
            \_ -> Expect.equal True opened.open
        , test "closed: label is 'show dates/times', no showtimes rendered" <|
            \_ ->
                view start
                    |> Query.fromHtml
                    |> Expect.all
                        [ Query.has [ text "show dates/times" ]
                        , Query.hasNot [ class "showtimes" ]
                        ]
        , test "open: label is 'hide dates/times' and the showtime link renders" <|
            \_ ->
                view opened
                    |> Query.fromHtml
                    |> Expect.all
                        [ Query.has [ text "hide dates/times" ]
                        , Query.has [ class "showtimes" ]
                        , Query.has [ class "time", text "4:00 PM" ]
                        ]
        ]
