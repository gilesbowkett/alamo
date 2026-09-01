module MainTest exposing (suite)

import Expect
import Json.Encode as E
import Main exposing (Msg(..), init, update, view)
import Test exposing (Test, describe, test)
import Test.Html.Query as Query
import Test.Html.Selector exposing (class, text)
import Toggle


slot : String -> E.Value
slot label =
    E.object [ ( "label", E.string label ), ( "href", E.string "https://example.com/x?cinemaId=1701&date=2026-09-05" ) ]


day : String -> String -> E.Value
day date label =
    E.object [ ( "date", E.string date ), ( "times", E.list identity [ slot label ] ) ]


film : String -> String -> String -> E.Value
film id title timeLabel =
    E.object
        [ ( "id", E.string id )
        , ( "title", E.string title )
        , ( "hero", E.string ("https://img/" ++ id ++ ".jpg") )
        , ( "url", E.string ("https://drafthouse.com/los-angeles/show/" ++ id ++ "?cinemaId=1701") )
        , ( "rt", E.string ("https://www.rottentomatoes.com/search?search=" ++ id) )
        , ( "days", E.list identity [ day "Sat Sep 5" timeLabel ] )
        ]


flags : E.Value
flags =
    E.object [ ( "films", E.list identity [ film "a" "Alpha" "4:00 PM", film "b" "Beta" "9:00 PM" ] ) ]


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
        , test "RemoveFilm drops that film, keeps the other" <|
            \_ ->
                Tuple.first (update (RemoveFilm "a") start)
                    |> view
                    |> Query.fromHtml
                    |> Expect.all
                        [ Query.hasNot [ text "Alpha" ]
                        , Query.has [ text "Beta" ]
                        ]
        , test "ToggleMsg opens only that film's showtimes" <|
            \_ ->
                Tuple.first (update (ToggleMsg "a" Toggle.Toggle) start)
                    |> view
                    |> Query.fromHtml
                    |> Expect.all
                        [ Query.has [ class "time", text "4:00 PM" ]
                        , Query.hasNot [ text "9:00 PM" ]
                        ]
        ]
