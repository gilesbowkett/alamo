module ShowtimesTest exposing (suite)

import Expect
import Showtimes exposing (Msg(..), Scope(..), init, update, view)
import Test exposing (Test, describe, test)
import Test.Html.Query as Query
import Test.Html.Selector exposing (class, text)


sampleDays : List Showtimes.Day
sampleDays =
    [ Showtimes.Day "2026-09-05" "Sat Sep 5" [ Showtimes.Slot "4:00 PM" "https://drafthouse.com/x?date=2026-09-05" ]
    , Showtimes.Day "2026-09-06" "Sun Sep 6" [ Showtimes.Slot "7:00 PM" "https://drafthouse.com/x?date=2026-09-06" ]
    ]


start : Showtimes.Model
start =
    init "1 hour 30 minutes" sampleDays


opened : Showtimes.Model
opened =
    update Toggle start


suite : Test
suite =
    describe "Showtimes component"
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
        , test "open (AllDays): label 'showtimes', runtime, date headers, and both days' times" <|
            \_ ->
                view opened
                    |> Query.fromHtml
                    |> Expect.all
                        [ Query.has [ text "showtimes" ]
                        , Query.has [ class "showtimes" ]
                        , Query.has [ class "runtime", text "1 hour 30 minutes" ]
                        , Query.has [ class "date", text "Sat Sep 5" ]
                        , Query.has [ class "time", text "4:00 PM" ]
                        , Query.has [ class "time", text "7:00 PM" ]
                        ]
        , test "open with no runtime: no runtime element" <|
            \_ ->
                update Toggle (init "" sampleDays)
                    |> view
                    |> Query.fromHtml
                    |> Query.hasNot [ class "runtime" ]
        , test "SingleDay closed: a showtimes link, but no times/runtime/date yet" <|
            \_ ->
                view { start | scope = SingleDay "2026-09-05" }
                    |> Query.fromHtml
                    |> Expect.all
                        [ Query.has [ class "toggle", text "showtimes" ]
                        , Query.hasNot [ class "time" ]
                        , Query.hasNot [ class "runtime" ]
                        , Query.hasNot [ class "date" ]
                        ]
        , test "SingleDay open: only the matching day's times + runtime, still no date header" <|
            \_ ->
                view { start | scope = SingleDay "2026-09-05", open = True }
                    |> Query.fromHtml
                    |> Expect.all
                        [ Query.has [ class "toggle", text "showtimes" ]
                        , Query.has [ class "time", text "4:00 PM" ]
                        , Query.hasNot [ text "7:00 PM" ]
                        , Query.has [ class "runtime", text "1 hour 30 minutes" ]
                        , Query.hasNot [ class "date" ]
                        ]
        ]
