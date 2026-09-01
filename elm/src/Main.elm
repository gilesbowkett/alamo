module Main exposing (Film, Model, Msg(..), init, main, update, view)

import Browser
import Dict exposing (Dict)
import Html exposing (Html, a, article, button, div, h2, img, text)
import Html.Attributes exposing (alt, class, href, rel, src, target)
import Html.Events exposing (onClick)
import Json.Decode as D
import Toggle


type alias Film =
    { title : String
    , hero : String
    , url : String
    , rt : String
    , toggle : Toggle.Model
    }


type alias Model =
    { order : List String
    , films : Dict String Film
    }


type Msg
    = RemoveFilm String
    | ToggleMsg String Toggle.Msg



-- INIT / FLAGS


filmDecoder : D.Decoder ( String, Film )
filmDecoder =
    D.map6
        (\id title hero url rt days ->
            ( id, Film title hero url rt (Toggle.init days) )
        )
        (D.field "id" D.string)
        (D.field "title" D.string)
        (D.field "hero" D.string)
        (D.field "url" D.string)
        (D.field "rt" D.string)
        Toggle.daysDecoder


flagsDecoder : D.Decoder (List ( String, Film ))
flagsDecoder =
    D.field "films" (D.list filmDecoder)


init : D.Value -> ( Model, Cmd Msg )
init flags =
    let
        pairs =
            Result.withDefault [] (D.decodeValue flagsDecoder flags)
    in
    ( { order = List.map Tuple.first pairs
      , films = Dict.fromList pairs
      }
    , Cmd.none
    )



-- UPDATE


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        RemoveFilm id ->
            ( { model
                | films = Dict.remove id model.films
                , order = List.filter (\i -> i /= id) model.order
              }
            , Cmd.none
            )

        ToggleMsg id sub ->
            ( { model
                | films =
                    Dict.update id (Maybe.map (\f -> { f | toggle = Toggle.update sub f.toggle })) model.films
              }
            , Cmd.none
            )



-- VIEW


view : Model -> Html Msg
view model =
    div []
        (List.filterMap (\id -> Maybe.map (viewFilm id) (Dict.get id model.films)) model.order)


viewFilm : String -> Film -> Html Msg
viewFilm id film =
    article [ class "film" ]
        [ div [ class "hero-wrap" ]
            [ a [ class "hero-link", href film.url, target "_blank", rel "noopener" ]
                [ img [ class "hero", src film.hero, alt film.title ] [] ]
            , button [ class "remove", onClick (RemoveFilm id) ] [ text "\u{00D7}" ]
            ]
        , div [ class "film-main" ]
            [ h2 []
                [ a [ href film.url, target "_blank", rel "noopener" ] [ text film.title ] ]
            , a [ class "rt", href film.rt, target "_blank", rel "noopener" ]
                [ text "check rotten tomatoes" ]
            , Html.map (ToggleMsg id) (Toggle.view film.toggle)
            ]
        ]


main : Program D.Value Model Msg
main =
    Browser.element
        { init = init
        , update = update
        , view = view
        , subscriptions = \_ -> Sub.none
        }
