module Showtimes exposing (Day, Model, Msg(..), Scope(..), Slot, init, update, view)

import Html exposing (Html, a, div, span, text)
import Html.Attributes exposing (class, href, rel, target)
import Html.Events exposing (onClick)


type alias Slot =
    { label : String, href : String }


type alias Day =
    { key : String, date : String, times : List Slot }


{-| Whether this showtimes list renders every day (list mode) or only one given
day (calendar mode, scoped by day `key`).
-}
type Scope
    = AllDays
    | SingleDay String


type alias Model =
    { open : Bool, runtime : String, days : List Day, scope : Scope }


type Msg
    = Toggle


init : String -> List Day -> Model
init runtime days =
    { open = False, runtime = runtime, days = days, scope = AllDays }


update : Msg -> Model -> Model
update msg model =
    case msg of
        Toggle ->
            { model | open = not model.open }


view : Model -> Html Msg
view model =
    case model.scope of
        AllDays ->
            let
                rest =
                    if model.open then
                        runtimeNode model.runtime ++ [ viewAllDays model.days ]

                    else
                        []
            in
            div [ class "contents" ]
                (a [ class "toggle", onClick Toggle ] [ text "showtimes" ] :: rest)

        SingleDay key ->
            -- Just the matching day's time links — no date header, toggle, or
            -- runtime. The calendar supplies the day's label and the film's title.
            case List.filter (\d -> d.key == key) model.days of
                day :: _ ->
                    div [ class "times" ] (List.map viewSlot day.times)

                [] ->
                    text ""


{-| The runtime sits inline on the links row, right after the toggle; omitted
entirely when unknown. Rendered only while open (it rides in `rest`).
-}
runtimeNode : String -> List (Html Msg)
runtimeNode runtime =
    if runtime == "" then
        []

    else
        [ span [ class "runtime" ] [ text runtime ] ]


viewAllDays : List Day -> Html Msg
viewAllDays days =
    div [ class "showtimes" ] (List.map viewDay days)


viewDay : Day -> Html Msg
viewDay day =
    div [ class "day" ]
        [ div [ class "date" ] [ text day.date ]
        , div [ class "times" ] (List.map viewSlot day.times)
        ]


viewSlot : Slot -> Html Msg
viewSlot slot =
    a [ class "time", href slot.href, target "_blank", rel "noopener" ] [ text slot.label ]
