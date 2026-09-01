module Main exposing (Day, Model, Msg(..), Slot, init, main, update, view)

import Browser
import Html exposing (Html, a, div, strong, text)
import Html.Attributes exposing (class, href, rel, target)
import Html.Events exposing (onClick)
import Json.Decode as D


type alias Slot =
    { label : String, href : String }


type alias Day =
    { date : String, times : List Slot }


type alias Model =
    { open : Bool, days : List Day }


type Msg
    = Toggle


slotDecoder : D.Decoder Slot
slotDecoder =
    D.map2 Slot (D.field "label" D.string) (D.field "href" D.string)


dayDecoder : D.Decoder Day
dayDecoder =
    D.map2 Day (D.field "date" D.string) (D.field "times" (D.list slotDecoder))


daysDecoder : D.Decoder (List Day)
daysDecoder =
    D.field "days" (D.list dayDecoder)


init : D.Value -> ( Model, Cmd Msg )
init flags =
    ( { open = False
      , days = Result.withDefault [] (D.decodeValue daysDecoder flags)
      }
    , Cmd.none
    )


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        Toggle ->
            ( { model | open = not model.open }, Cmd.none )


view : Model -> Html Msg
view model =
    let
        label =
            if model.open then
                "hide dates/times"

            else
                "show dates/times"

        rest =
            if model.open then
                [ viewShowtimes model.days ]

            else
                []
    in
    div [ class "contents" ]
        (a [ class "toggle", onClick Toggle ] [ text label ] :: rest)


viewShowtimes : List Day -> Html Msg
viewShowtimes days =
    div [ class "showtimes" ] (List.map viewDay days)


viewDay : Day -> Html Msg
viewDay day =
    div [ class "day" ]
        [ strong [ class "date" ] [ text day.date ]
        , div [ class "times" ] (List.map viewSlot day.times)
        ]


viewSlot : Slot -> Html Msg
viewSlot slot =
    a [ class "time", href slot.href, target "_blank", rel "noopener" ] [ text slot.label ]


main : Program D.Value Model Msg
main =
    Browser.element
        { init = init
        , update = update
        , view = view
        , subscriptions = \_ -> Sub.none
        }
