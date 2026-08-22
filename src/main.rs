use bevy::app::{App, AppExit, PluginGroup, Startup, Update};
use bevy::ecs::{
    message::MessageReader,
    observer::On,
    query::Changed,
    schedule::{common_conditions::{any_match_filter, on_message}, IntoScheduleConfigs, SystemCondition},
    system::{Commands, Query, Res, ResMut}
};
use bevy::input::{keyboard::KeyCode, ButtonInput};
use bevy::log::info;
use bevy::picking::events::{Click, Pointer};
use bevy::utils::default;
use bevy::DefaultPlugins;
use bevy::window::{Window, WindowPlugin};

use crate::cards::{Suit, Value};
use crate::fireworks::{animate_fireworks, expire_fireworks, launch_fireworks};
use crate::game::{check_for_victory, check_guesses, clear_guesses, guess_suit, guess_value, redeal_game, select_tile, solve_all, Clue, GameMessage, GameSeed, LayoutData, Selection, Tile};
use crate::layout::setup_layout;
use crate::render::RenderPlugin;

mod cards;
mod deal;
mod fireworks;
mod game;
mod layout;
mod poker;
mod render;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            fit_canvas_to_parent: true,
            ..default()
        }),
        ..default()
    }));

    app.add_message::<GameMessage>();
    app.init_resource::<GameSeed>();
    app.init_resource::<LayoutData>();
    app.init_resource::<Selection>();
    app.add_systems(Startup, setup_layout);
    app.add_systems(Startup, redeal_game.after(setup_layout));
    app.add_systems(Update, handle_input);
    app.add_systems(Update, handle_game_messages.run_if(on_message::<GameMessage>));
    app.add_systems(Update, check_guesses.run_if(any_match_filter::<Changed<Tile>>));
    app.add_systems(Update, check_for_victory.run_if(any_match_filter::<Changed<Clue>>.or_else(any_match_filter::<Changed<Tile>>)));
    app.add_systems(Update, (animate_fireworks, expire_fireworks).chain());
    app.add_observer(on_click);

    app.add_plugins(RenderPlugin);

    app.run();
}

fn handle_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
) {
    // Only check for exiting if we're not running as a web app
    #[cfg(not(target_arch = "wasm32"))]
    if keyboard_input.just_pressed(KeyCode::Escape) {
        commands.write_message(GameMessage::Quit);
    }

    if keyboard_input.just_pressed(KeyCode::KeyR) {
        commands.write_message(GameMessage::Redeal);
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
    } else if keyboard_input.any_just_pressed([KeyCode::Space, KeyCode::Backspace, KeyCode::Delete]) {
        commands.write_message(GameMessage::ClearGuesses);
    } else if keyboard_input.just_pressed(KeyCode::KeyX) {
        commands.write_message(GameMessage::SolveAll);
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

    selection.position = Some(tile.position);

    commands.write_message(GameMessage::SelectTile);
}

fn handle_game_messages(
    mut messages: MessageReader<GameMessage>,
    mut commands: Commands,
) {
    for message in messages.read() {
        info!("Handling: {message:?}");
        match message {
            GameMessage::Redeal => commands.run_system_cached(redeal_game),
            GameMessage::Quit => _ = commands.write_message(AppExit::Success),
            GameMessage::SelectTile => commands.run_system_cached(select_tile),
            GameMessage::GuessSuit(suit) => commands.run_system_cached_with(guess_suit, *suit),
            GameMessage::GuessValue(value) => commands.run_system_cached_with(guess_value, *value),
            GameMessage::ClearGuesses => commands.run_system_cached(clear_guesses),
            GameMessage::SolveAll => commands.run_system_cached(solve_all),
            GameMessage::Victory => commands.run_system_cached(launch_fireworks),
        }
    }
}
