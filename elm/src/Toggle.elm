module Toggle exposing (Day, Model, Msg(..), Slot, daysDecoder, init, update, view)

import Html exposing (Html, a, div, text)
import Html.Attributes exposing (class, href, rel, target)
import Html.Events exposing (onClick)
import Json.Decode as D


type alias Slot =
    { label : String, href : String }


type alias Day =
    { date : String, times : List Slot }


type alias Model =
    { open : Bool, runtime : String, days : List Day }


type Msg
    = Toggle


init : String -> List Day -> Model
init runtime days =
    { open = False, runtime = runtime, days = days }


update : Msg -> Model -> Model
update msg model =
    case msg of
        Toggle ->
            { model | open = not model.open }


view : Model -> Html Msg
view model =
    let
        rest =
            if model.open then
                [ viewShowtimes model.runtime model.days ]

            else
                []
    in
    div [ class "contents" ]
        (a [ class "toggle", onClick Toggle ] [ text "Showtimes." ] :: rest)


viewShowtimes : String -> List Day -> Html Msg
viewShowtimes runtime days =
    div [ class "showtimes" ]
        (viewRuntime runtime ++ List.map viewDay days)


{-| The runtime sits above the stack of day rows; omitted entirely when unknown.
-}
viewRuntime : String -> List (Html Msg)
viewRuntime runtime =
    if runtime == "" then
        []

    else
        [ div [ class "runtime" ] [ text runtime ] ]


viewDay : Day -> Html Msg
viewDay day =
    div [ class "day" ]
        [ div [ class "date" ] [ text day.date ]
        , div [ class "times" ] (List.map viewSlot day.times)
        ]


viewSlot : Slot -> Html Msg
viewSlot slot =
    a [ class "time", href slot.href, target "_blank", rel "noopener" ] [ text slot.label ]



-- Decoders (used by the container to build each film's toggle from flags)


slotDecoder : D.Decoder Slot
slotDecoder =
    D.map2 Slot (D.field "label" D.string) (D.field "href" D.string)


dayDecoder : D.Decoder Day
dayDecoder =
    D.map2 Day (D.field "date" D.string) (D.field "times" (D.list slotDecoder))


daysDecoder : D.Decoder (List Day)
daysDecoder =
    D.field "days" (D.list dayDecoder)
