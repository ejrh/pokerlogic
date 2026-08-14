use bevy::app::{App, AppExit, Startup, Update};
use bevy::asset::{AssetServer, Handle};
use bevy::camera::Camera2d;
use bevy::color::Color;
use bevy::DefaultPlugins;
use bevy::ecs::system::Commands;
use bevy::input::ButtonInput;
use bevy::prelude::{on_message, Bundle, ChildOf, Click, Entity, FlexDirection, GridTrack, IntoScheduleConfigs, JustifyContent, KeyCode, Message, MessageReader, On, Pointer, Query, Res, ResMut, Resource};
use bevy::text::{Font, FontSize, TextColor, TextFont};
use bevy::ui::{percent, widget::Text, AlignContent, AlignItems, AlignSelf, BackgroundColor, BorderColor, Display, FocusPolicy, JustifySelf, MaxTrackSizingFunction, MinTrackSizingFunction, Node, UiRect, Val};
use bevy::utils::default;

use crate::cards::{Suit, Value};
use crate::render::render_game;
use crate::game::{Tile, restart_game, select_tile, Selection, guess_suit, guess_value, clear_guesses};

mod cards;
mod poker;
mod game;
mod render;

#[derive(Resource)]
struct LayoutData {
    font: Handle<Font>,
    column_ids: Vec<Entity>,
    row_ids: Vec<Entity>,
    tile_ids: Vec<Vec<Entity>>,
}

impl Default for LayoutData {
    fn default() -> Self {
        LayoutData {
            font: Handle::default(),
            column_ids: vec![Entity::PLACEHOLDER; 5],
            row_ids: vec![Entity::PLACEHOLDER; 5],
            tile_ids: vec![vec![Entity::PLACEHOLDER; 5]; 5],
        }
    }
}

#[derive(Message)]
enum GameMessage {
    Restart,
    Quit,
    SelectTile,
    GuessSuit(Suit),
    GuessValue(Value),
    ClearGuesses,
}

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins);

    app.add_message::<GameMessage>();
    app.init_resource::<LayoutData>();
    app.init_resource::<Selection>();
    app.add_systems(Startup, setup_layout);
    app.add_systems(Startup, restart_game.after(setup_layout));
    app.add_systems(Update, handle_input);
    app.add_systems(Update, handle_game_messages.run_if(on_message::<GameMessage>));
    app.add_systems(Update, render_game.after(handle_game_messages));
    app.add_observer(on_click);

    app.run();
}

fn setup_layout(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut data: ResMut<LayoutData>,
) {
    data.font = asset_server.load("fonts/FiraMono-Medium.ttf");

    commands.spawn(Camera2d);

    let grid_template_columns = vec![
        GridTrack::fr(0.8),
        GridTrack::fr(1.0),
        GridTrack::fr(1.0),
        GridTrack::fr(1.0),
        GridTrack::fr(1.0),
        GridTrack::fr(1.0),
    ];

    let grid_template_rows = vec![
        GridTrack::minmax(MinTrackSizingFunction::Px(80.0), MaxTrackSizingFunction::Auto),
        GridTrack::fr(1.0),
        GridTrack::fr(1.0),
        GridTrack::fr(1.0),
        GridTrack::fr(1.0),
        GridTrack::fr(1.0),
    ];

    let parent_id = commands.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            width: percent(100),
            height: percent(100),
            align_content: AlignContent::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
    )).id();

    let board_id = commands.spawn((
        Node {
            width: percent(100),
            height: percent(100),
            display: Display::Grid,
            align_self: AlignSelf::Center,
            justify_self: JustifySelf::Center,
            grid_template_columns,
            grid_template_rows,
            ..default()
        },
        BackgroundColor(Color::srgb(0.1, 0.2, 0.1)),
        ChildOf(parent_id),
    )).id();

    fn make_heading() -> impl Bundle {
        Node {
            display: Display::Flex,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(8.0)),
            ..default()
        }
    }

    fn make_card() -> impl Bundle {
        (
            Node {
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.0)),
                margin: UiRect::all(Val::Px(8.0)),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BorderColor::all(Color::srgb(0.0, 0.5, 0.0)),
            FocusPolicy::Block,
        )
    }

    commands.spawn((
        Node::default(), ChildOf(board_id),
    ));

    for i in 0..5 {
        data.column_ids[i] = commands.spawn((
            make_heading(),
            ChildOf(board_id),
        )).id();
    }

    for i in 0..5 {
        data.row_ids[i] = commands.spawn((
            make_heading(),
            ChildOf(board_id),
        )).id();
        for j in 0..5 {
            data.tile_ids[i][j] = commands.spawn((
                make_card(),
                ChildOf(board_id),
            )).id();
        }
    }

    // Instructions at bottom
    commands.spawn((
        Node {
            ..default()
        },
        ChildOf(parent_id),
        Text::new("R - restart; Click to guess specific card"),
        TextColor(Color::srgb(0.8, 0.6, 0.6)),
        TextFont::from(data.font.clone()).with_font_size(FontSize::Px(24.0)),
    ));
    commands.spawn((
        Node {
            ..default()
        },
        ChildOf(parent_id),
        Text::new("(C,D,H,S) - guess suit; (2-9,1) - guess value; Space - clear guess"),
        TextColor(Color::srgb(0.8, 0.6, 0.6)),
        TextFont::from(data.font.clone()).with_font_size(FontSize::Px(24.0)),
    ));
}

fn handle_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
) {
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        commands.write_message(GameMessage::Restart);
    } else if keyboard_input.just_pressed(KeyCode::Escape) {
        commands.write_message(GameMessage::Quit);
    } else if keyboard_input.just_pressed(KeyCode::KeyC) {
        commands.write_message(GameMessage::GuessSuit(Suit::Clubs));
    } else if keyboard_input.just_pressed(KeyCode::KeyH) {
        commands.write_message(GameMessage::GuessSuit(Suit::Hearts));
    } else if keyboard_input.just_pressed(KeyCode::KeyD) {
        commands.write_message(GameMessage::GuessSuit(Suit::Diamonds));
    } else if keyboard_input.just_pressed(KeyCode::KeyS) {
        commands.write_message(GameMessage::GuessSuit(Suit::Spades));
    } else if keyboard_input.just_pressed(KeyCode::Digit0) {
        commands.write_message(GameMessage::GuessValue(Value::of_number(10)));
    } else if keyboard_input.just_pressed(KeyCode::Digit1) {
        commands.write_message(GameMessage::GuessValue(Value::of_number(10)));
    } else if keyboard_input.just_pressed(KeyCode::Digit2) {
        commands.write_message(GameMessage::GuessValue(Value::of_number(2)));
    } else if keyboard_input.just_pressed(KeyCode::Digit3) {
        commands.write_message(GameMessage::GuessValue(Value::of_number(3)));
    } else if keyboard_input.just_pressed(KeyCode::Digit4) {
        commands.write_message(GameMessage::GuessValue(Value::of_number(4)));
    } else if keyboard_input.just_pressed(KeyCode::Digit5) {
        commands.write_message(GameMessage::GuessValue(Value::of_number(5)));
    } else if keyboard_input.just_pressed(KeyCode::Digit6) {
        commands.write_message(GameMessage::GuessValue(Value::of_number(6)));
    } else if keyboard_input.just_pressed(KeyCode::Digit7) {
        commands.write_message(GameMessage::GuessValue(Value::of_number(7)));
    } else if keyboard_input.just_pressed(KeyCode::Digit8) {
        commands.write_message(GameMessage::GuessValue(Value::of_number(8)));
    } else if keyboard_input.just_pressed(KeyCode::Digit9) {
        commands.write_message(GameMessage::GuessValue(Value::of_number(9)));
    } else if keyboard_input.just_pressed(KeyCode::KeyJ) {
        commands.write_message(GameMessage::GuessValue(Value::of_face('J')));
    } else if keyboard_input.just_pressed(KeyCode::KeyQ) {
        commands.write_message(GameMessage::GuessValue(Value::of_face('Q')));
    } else if keyboard_input.just_pressed(KeyCode::KeyK) {
        commands.write_message(GameMessage::GuessValue(Value::of_face('K')));
    } else if keyboard_input.just_pressed(KeyCode::KeyA) {
        commands.write_message(GameMessage::GuessValue(Value::of_face('A')));
    } else if keyboard_input.just_pressed(KeyCode::Space) {
        commands.write_message(GameMessage::ClearGuesses);
    }
}

fn on_click(
    click: On<Pointer<Click>>,
    tiles: Query<&Tile>,
    mut selection: ResMut<Selection>,
    mut commands: Commands,
) {
    let Ok(tile) = tiles.get(click.entity)
    else { return; };

    println!("Clicked tile");

    selection.position = tile.position;

    commands.write_message(GameMessage::SelectTile);
}

fn handle_game_messages(
    mut messages: MessageReader<GameMessage>,
    mut commands: Commands,
) {
    for message in messages.read() {
        match message {
            GameMessage::Restart => commands.run_system_cached(restart_game),
            GameMessage::Quit => _ = commands.write_message(AppExit::Success),
            GameMessage::SelectTile => commands.run_system_cached(select_tile),
            GameMessage::GuessSuit(suit) => commands.run_system_cached_with(guess_suit, *suit),
            GameMessage::GuessValue(value) => commands.run_system_cached_with(guess_value, *value),
            GameMessage::ClearGuesses => commands.run_system_cached(clear_guesses),
        }
    }
}
