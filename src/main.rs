use bevy::app::{App, AppExit, Startup, Update};
use bevy::asset::AssetServer;
use bevy::camera::visibility::RenderLayers;
use bevy::camera::{Camera, Camera2d, ClearColorConfig};
use bevy::color::Color;
use bevy::ecs::{bundle::Bundle, hierarchy::ChildOf, message::MessageReader, observer::On, query::Changed, schedule::{common_conditions::{any_match_filter, on_message}, IntoScheduleConfigs}, system::{Commands, Query, Res, ResMut}};
use bevy::input::{keyboard::KeyCode, ButtonInput};
use bevy::log::info;
use bevy::picking::events::{Click, Pointer};
use bevy::text::{FontSize, TextColor, TextFont};
use bevy::ui::{percent, widget::Text, AlignContent, AlignItems, AlignSelf, BackgroundColor, BorderRadius, Display, FlexDirection, FocusPolicy, GridPlacement, GridTrack, IsDefaultUiCamera, JustifyContent, JustifySelf, MaxTrackSizingFunction, MinTrackSizingFunction, Node, UiRect, Val};
use bevy::utils::default;
use bevy::DefaultPlugins;
use bevy::prelude::resource_changed;

use crate::cards::{Suit, Value};
use crate::fireworks::{animate_fireworks, expire_fireworks, launch_fireworks};
use crate::game::{check_for_victory, check_guesses, clear_guesses, guess_suit, guess_value, redeal_game, select_tile, solve_all, Clue, GameMessage, GameSeed, LayoutData, Selection, Tile, CLUE_INDICES};
use crate::render::{render_clues, render_game_seed, render_tiles, GameSeedLabel};

mod cards;
mod fireworks;
mod game;
mod poker;
mod render;
mod deal;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins);

    app.add_message::<GameMessage>();
    app.init_resource::<GameSeed>();
    app.init_resource::<LayoutData>();
    app.init_resource::<Selection>();
    app.add_systems(Startup, setup_layout);
    app.add_systems(Startup, redeal_game.after(setup_layout));
    app.add_systems(Update, handle_input);
    app.add_systems(Update, handle_game_messages.run_if(on_message::<GameMessage>));
    app.add_systems(Update, (render_tiles, render_clues).after(handle_game_messages));
    app.add_systems(Update, render_game_seed.run_if(resource_changed::<GameSeed>).after(handle_game_messages));
    app.add_systems(Update, check_guesses.run_if(any_match_filter::<Changed<Tile>>));
    app.add_systems(Update, check_for_victory.run_if(any_match_filter::<Changed<Clue>>));
    app.add_systems(Update, (animate_fireworks, expire_fireworks).chain());
    app.add_observer(on_click);

    app.run();
}

fn setup_layout(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut data: ResMut<LayoutData>,
) {
    data.font = asset_server.load("fonts/FiraMono-Medium.ttf");
    data.symbol_font = asset_server.load("fonts/JetBrainsMono-Medium.ttf");

    commands.spawn((
        Camera2d,
        Camera {
            order: 0,
            ..default()
        },
        IsDefaultUiCamera,
    ));

    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        RenderLayers::layer(1),
    ));

    let grid_template_columns = vec![
        GridTrack::fr(1.2),
        GridTrack::fr(1.0),
        GridTrack::fr(1.0),
        GridTrack::fr(1.0),
        GridTrack::fr(1.0),
        GridTrack::fr(1.0),
        GridTrack::fr(1.0),
        GridTrack::fr(1.0),
        GridTrack::fr(1.0),
        GridTrack::fr(1.0),
        GridTrack::fr(1.0),
        GridTrack::fr(1.2),
    ];

    let grid_template_rows = vec![
        GridTrack::minmax(MinTrackSizingFunction::Px(80.0), MaxTrackSizingFunction::Auto),
        GridTrack::fr(1.0),
        GridTrack::fr(1.0),
        GridTrack::fr(1.0),
        GridTrack::fr(1.0),
        GridTrack::fr(1.0),
        GridTrack::minmax(MinTrackSizingFunction::Px(80.0), MaxTrackSizingFunction::Auto),
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

    fn make_heading(span: u16, topleft: bool) -> impl Bundle {
        let align_items;
        let justify_content;
        if topleft {
            align_items = AlignItems::Start;
            justify_content = JustifyContent::Start;
        } else {
            align_items = AlignItems::End;
            justify_content = JustifyContent::End;
        }
        Node {
            display: Display::Flex,
            align_items,
            justify_content,
            padding: UiRect::all(Val::Px(8.0)),
            grid_column: GridPlacement::span(span),
            ..default()
        }
    }

    fn make_card(topleft: bool) -> impl Bundle {
        let align_items;
        let justify_content;
        let border;
        let padding;
        let margin;
        let border_radius;
        if topleft {
            align_items = AlignItems::Start;
            justify_content = JustifyContent::Start;
            border = UiRect::top(Val::Px(2.0)).with_left(Val::Px(2.0));
            padding = UiRect::all(Val::Px(8.0)).with_right(Val::Px(0.0));
            margin = UiRect::right(Val::Px(2.0)).with_bottom(Val::Px(40.0));
            border_radius = BorderRadius::top_left(Val::Px(10.0))
        } else {
            align_items = AlignItems::End;
            justify_content = JustifyContent::End;
            border = UiRect::bottom(Val::Px(2.0)).with_right(Val::Px(2.0));
            padding = UiRect::all(Val::Px(8.0)).with_left(Val::Px(0.0));
            margin = UiRect::left(Val::Px(2.0)).with_top(Val::Px(40.0));
            border_radius = BorderRadius::bottom_right(Val::Px(10.0))
        };

        (
            Node {
                display: Display::Flex,
                align_items,
                justify_content,
                border,
                margin,
                padding,
                border_radius,
                ..default()
            },
            FocusPolicy::Block,
        )
    }

    fn make_spare() -> impl Bundle {
        (
            Node {
                display: Display::Flex,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(Val::Px(2.0)).with_bottom(Val::Px(0.0)),
                padding: UiRect::all(Val::Px(8.0)),
                margin: UiRect::all(Val::Px(4.0)),
                border_radius: BorderRadius::top(Val::Px(10.0)),
                ..default()
            },
            FocusPolicy::Block,
        )
    }

    commands.spawn((
        Node::default(), ChildOf(board_id),
    ));

    for i in CLUE_INDICES {
        data.top_ids[i] = commands.spawn((
            make_heading(2, true),
            ChildOf(board_id),
        )).id();
    }

    commands.spawn((
        Node::default(),
        ChildOf(board_id),
        GameSeedLabel,
    ));

    for i in CLUE_INDICES {
        data.left_ids[i] = commands.spawn((
            make_heading(1, true),
            ChildOf(board_id),
        )).id();
        for j in CLUE_INDICES {
            data.tile_ids[i][j][0] = commands.spawn((
                make_card(true),
                ChildOf(board_id),
            )).id();
            data.tile_ids[i][j][1] = commands.spawn((
                make_card(false),
                ChildOf(board_id),
            )).id();
        }
        data.right_ids[i] = commands.spawn((
            make_heading(1, false),
            ChildOf(board_id),
        )).id();
    }

    data.spare_ids[0] =commands.spawn((
        make_spare(),
        ChildOf(board_id),
    )).id();

    for i in CLUE_INDICES {
        data.bottom_ids[i] = commands.spawn((
            make_heading(2, false),
            ChildOf(board_id),
        )).id();
    }

    data.spare_ids[1] =commands.spawn((
        make_spare(),
        ChildOf(board_id),
    )).id();

    // Instructions at bottom
    commands.spawn((
        Node {
            ..default()
        },
        ChildOf(parent_id),
        Text::new("R - redeal; Click to guess specific card"),
        TextColor(Color::srgb(0.8, 0.6, 0.6)),
        TextFont::from(data.font.clone()).with_font_size(FontSize::Px(24.0)),
    ));
    commands.spawn((
        Node {
            ..default()
        },
        ChildOf(parent_id),
        Text::new("(C,D,H,S) - guess suit; (2-10,J,Q,K,A) - guess value; Space - clear guess"),
        TextColor(Color::srgb(0.8, 0.6, 0.6)),
        TextFont::from(data.font.clone()).with_font_size(FontSize::Px(24.0)),
    ));
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
