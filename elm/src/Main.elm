module Main exposing (Day, Film, Model, Msg(..), Tab(..), init, main, update, view)

import Browser
import Dict exposing (Dict)
import Html exposing (Html, a, article, button, div, h2, img, span, text)
import Html.Attributes exposing (alt, class, classList, href, rel, src, target)
import Html.Events exposing (onClick)
import Json.Decode as D
import Set exposing (Set)
import Showtimes
import Svg
import Svg.Attributes as SA


type alias Film =
    { title : String
    , hero : String
    , url : String
    , rt : String
    , showtimes : Showtimes.Model
    , visible : Bool
    }


type alias Day =
    { key : String
    , date : String
    , ghosted : Bool
    , filmIds : List Int
    }


type Tab
    = ListTab
    | CalendarTab


type alias Model =
    { films : Dict Int Film
    , calendar : List Day
    , tab : Tab

    -- (filmId, dayKey) pairs whose showtimes are expanded in the calendar. Per
    -- cell, so a film's days (and list mode) toggle independently.
    , calendarOpen : Set ( Int, String )
    }


type Msg
    = HideFilm Int
    | ShowtimesMsg Int Showtimes.Msg
    | ToggleCalendarFilm Int String
    | SetTab Tab



-- INIT


filmDecoder : D.Decoder ( Int, Film )
filmDecoder =
    D.map7
        (\id title hero url rt runtime days ->
            ( id
            , { title = title
              , hero = hero
              , url = url
              , rt = rt
              , showtimes = Showtimes.init runtime days
              , visible = True
              }
            )
        )
        (D.field "id" D.int)
        (D.field "title" D.string)
        (D.field "hero" D.string)
        (D.field "url" D.string)
        (D.field "rt" D.string)
        (D.field "runtime" D.string)
        (D.field "days" (D.list showtimeDayDecoder))


slotDecoder : D.Decoder Showtimes.Slot
slotDecoder =
    D.map2 Showtimes.Slot (D.field "label" D.string) (D.field "href" D.string)


showtimeDayDecoder : D.Decoder Showtimes.Day
showtimeDayDecoder =
    D.map3 Showtimes.Day
        (D.field "key" D.string)
        (D.field "date" D.string)
        (D.field "times" (D.list slotDecoder))


calendarDayDecoder : D.Decoder Day
calendarDayDecoder =
    D.map4 Day
        (D.field "key" D.string)
        (D.field "date" D.string)
        (D.field "ghosted" D.bool)
        (D.field "filmIds" (D.list D.int))


init : D.Value -> ( Model, Cmd Msg )
init flags =
    let
        pairs =
            Result.withDefault [] (D.decodeValue (D.field "films" (D.list filmDecoder)) flags)

        calendar =
            Result.withDefault [] (D.decodeValue (D.field "calendar" (D.list calendarDayDecoder)) flags)
    in
    ( { films = Dict.fromList pairs
      , calendar = calendar
      , tab = ListTab
      , calendarOpen = Set.empty
      }
    , Cmd.none
    )



-- UPDATE


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        HideFilm id ->
            ( { model
                | films = Dict.update id (Maybe.map (\f -> { f | visible = False })) model.films
              }
            , Cmd.none
            )

        ShowtimesMsg id sub ->
            ( { model
                | films =
                    Dict.update id (Maybe.map (\f -> { f | showtimes = Showtimes.update sub f.showtimes })) model.films
              }
            , Cmd.none
            )

        ToggleCalendarFilm id key ->
            let
                cell =
                    ( id, key )

                calendarOpen =
                    if Set.member cell model.calendarOpen then
                        Set.remove cell model.calendarOpen

                    else
                        Set.insert cell model.calendarOpen
            in
            ( { model | calendarOpen = calendarOpen }, Cmd.none )

        SetTab tab ->
            ( { model | tab = tab }, Cmd.none )



-- VIEW


view : Model -> Html Msg
view model =
    div []
        [ viewTabs model.tab
        , case model.tab of
            ListTab ->
                div []
                    (Dict.toList model.films
                        |> List.filter (\( _, f ) -> f.visible)
                        |> List.map (\( id, f ) -> viewFilm id f)
                    )

            CalendarTab ->
                viewCalendar model.films model.calendarOpen model.calendar
        ]


viewTabs : Tab -> Html Msg
viewTabs tab =
    div [ class "tabs" ]
        [ tabButton (tab == ListTab) (SetTab ListTab) iconList "list"
        , tabButton (tab == CalendarTab) (SetTab CalendarTab) iconCalendar "calendar"
        ]


tabButton : Bool -> Msg -> Html Msg -> String -> Html Msg
tabButton active msg icon label =
    button [ class "tab", classList [ ( "active", active ) ], onClick msg ]
        [ icon, span [] [ text label ] ]


viewCalendar : Dict Int Film -> Set ( Int, String ) -> List Day -> Html Msg
viewCalendar films calendarOpen days =
    div [ class "calendar" ] (List.map (viewDay films calendarOpen) days)


viewDay : Dict Int Film -> Set ( Int, String ) -> Day -> Html Msg
viewDay films calendarOpen d =
    div [ class "calendar-day", classList [ ( "ghost", d.ghosted ) ] ]
        (div [ class "calendar-date" ] [ text d.date ]
            :: List.filterMap (viewCalendarFilm films calendarOpen d.key) d.filmIds
        )


{-| One film in a calendar cell: title above a tiny hero above a "showtimes"
link that toggles that day's times + runtime. The per-cell open state lives in
`calendarOpen`, so the link click routes to `ToggleCalendarFilm` (not the shared
`Film.showtimes`). Dropped when hidden or unknown, so a hidden film leaves too.
-}
viewCalendarFilm : Dict Int Film -> Set ( Int, String ) -> String -> Int -> Maybe (Html Msg)
viewCalendarFilm films calendarOpen key id =
    case Dict.get id films of
        Just film ->
            if film.visible then
                let
                    fs =
                        film.showtimes

                    scoped =
                        { fs | scope = Showtimes.SingleDay key, open = Set.member ( id, key ) calendarOpen }
                in
                Just <|
                    div [ class "calendar-film" ]
                        [ div [ class "calendar-title" ] [ text film.title ]
                        , div [ class "hero-wrap" ]
                            [ img [ class "calendar-hero", src film.hero, alt film.title ] []
                            , button [ class "remove", onClick (HideFilm id) ] [ text "\u{2717}" ]
                            ]
                        , Html.map (\_ -> ToggleCalendarFilm id key) (Showtimes.view scoped)
                        ]

            else
                Nothing

        Nothing ->
            Nothing


viewFilm : Int -> Film -> Html Msg
viewFilm id film =
    article [ class "film" ]
        [ div [ class "hero-wrap" ]
            [ a [ class "hero-link", href film.url, target "_blank", rel "noopener" ]
                [ img [ class "hero", src film.hero, alt film.title ] [] ]
            , button [ class "remove", onClick (HideFilm id) ] [ text "\u{2717}" ]
            ]
        , div [ class "film-main" ]
            [ h2 []
                [ a [ href film.url, target "_blank", rel "noopener" ] [ text film.title ] ]
            , a [ class "rt", href film.rt, target "_blank", rel "noopener" ]
                [ text "check rotten tomatoes" ]
            , Html.map (ShowtimesMsg id) (Showtimes.view film.showtimes)
            ]
        ]



iconList : Html msg
iconList =
    Svg.svg [ SA.class "icon", SA.viewBox "0 0 512 512", SA.width "1em", SA.height "1em" ]
        [ Svg.path [ SA.fill "currentColor", SA.d "M40 48C26.7 48 16 58.7 16 72l0 48c0 13.3 10.7 24 24 24l48 0c13.3 0 24-10.7 24-24l0-48c0-13.3-10.7-24-24-24L40 48zM192 64c-17.7 0-32 14.3-32 32s14.3 32 32 32l288 0c17.7 0 32-14.3 32-32s-14.3-32-32-32L192 64zm0 160c-17.7 0-32 14.3-32 32s14.3 32 32 32l288 0c17.7 0 32-14.3 32-32s-14.3-32-32-32l-288 0zm0 160c-17.7 0-32 14.3-32 32s14.3 32 32 32l288 0c17.7 0 32-14.3 32-32s-14.3-32-32-32l-288 0zM16 232l0 48c0 13.3 10.7 24 24 24l48 0c13.3 0 24-10.7 24-24l0-48c0-13.3-10.7-24-24-24l-48 0c-13.3 0-24 10.7-24 24zM40 368c-13.3 0-24 10.7-24 24l0 48c0 13.3 10.7 24 24 24l48 0c13.3 0 24-10.7 24-24l0-48c0-13.3-10.7-24-24-24l-48 0z" ] [] ]


iconCalendar : Html msg
iconCalendar =
    Svg.svg [ SA.class "icon", SA.viewBox "0 0 448 512", SA.width "1em", SA.height "1em" ]
        [ Svg.path [ SA.fill "currentColor", SA.d "M96 32l0 32L48 64C21.5 64 0 85.5 0 112l0 48 448 0 0-48c0-26.5-21.5-48-48-48l-48 0 0-32c0-17.7-14.3-32-32-32s-32 14.3-32 32l0 32L160 64l0-32c0-17.7-14.3-32-32-32S96 14.3 96 32zM448 192L0 192 0 464c0 26.5 21.5 48 48 48l352 0c26.5 0 48-21.5 48-48l0-272z" ] [] ]


main : Program D.Value Model Msg
main =
    Browser.element
        { init = init
        , update = update
        , view = view
        , subscriptions = \_ -> Sub.none
        }
