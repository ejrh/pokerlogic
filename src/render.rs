use bevy::app::{App, Plugin, PostUpdate, Startup};
use bevy::asset::{AssetServer, Handle};
use bevy::color::Color;
use bevy::ecs::{
    component::Component,
    entity::Entity,
    hierarchy::ChildOf,
    query::{Changed, With},
    resource::Resource,
    schedule::{common_conditions::{on_message, resource_changed}, IntoScheduleConfigs, SystemSet},
    system::{Commands, Query, Res, ResMut, Single},
};
use bevy::log::info;
use bevy::math::Vec2;
use bevy::text::{Font, FontSize, Justify, TextColor, TextFont, TextLayout};
use bevy::ui::{widget::Text, BackgroundColor, BorderColor, Node, PositionType, UiScale, Val};
use bevy::utils::default;
use bevy::window::{Window, WindowResized};

use crate::cards::SuitColour;
use crate::game::{Clue, ClueGuessState, ClueLocation, GameSeed, Tile};
use crate::{handle_game_messages, LayoutData};

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Theme>();

        app.add_systems(Startup, setup_theme.in_set(RenderSystems));

        app.add_systems(PostUpdate, (
                render_tiles,
                render_clues,
                render_game_seed.run_if(resource_changed::<GameSeed>),
                adjust_scaling.run_if(on_message::<WindowResized>),
            ).in_set(RenderSystems)
            .after(handle_game_messages)
        );
    }
}

#[derive(SystemSet, Clone, Debug, Eq, Hash, PartialEq)]
pub struct RenderSystems;

#[derive(Resource, Default)]
pub struct Theme {
    pub font: Handle<Font>,
    pub symbol_font: Handle<Font>,
}

#[derive(Component)]
pub struct GameSeedLabel;

fn setup_theme(
    asset_server: Res<AssetServer>,
    mut theme: ResMut<Theme>,
) {
    theme.font = asset_server.load("fonts/FiraMono-Medium.ttf");
    theme.symbol_font = asset_server.load("fonts/JetBrainsMono-Medium.ttf");
}

pub fn render_tiles(
    theme: Res<Theme>,
    layout_data: Res<LayoutData>,
    tiles: Query<&Tile, Changed<Tile>>,
    mut commands: Commands
) {
    fn render_tile(theme: &Theme, commands: &mut Commands, tile_id: Entity, tile: &Tile) {
        commands.entity(tile_id).despawn_children();

        let bg = if tile.known {
            Color::srgb(0.9, 0.9, 0.9)
        } else if tile.selected {
            Color::srgb(0.6, 0.9, 0.6)
        } else {
            Color::srgb(0.8, 0.8, 0.8)
        };

        let border_color = if tile.selected {
            Color::srgb(0.1, 0.8, 0.1)
        } else {
            Color::srgb(0.0, 0.0, 0.0)
        };

        commands.entity(tile_id)
            .insert((
                BackgroundColor(bg),
                BorderColor::all(border_color),
            ));

        let suit_str;
        let value_str;
        let colour;
        if tile.known {
            suit_str = tile.card.suit.symbol();
            value_str = tile.card.value.symbol();
            colour = match tile.card.suit.colour() {
                SuitColour::Red => Color::srgb(0.8, 0.1, 0.1),
                SuitColour::Black => Color::srgb(0.1, 0.1, 0.1),
            };
        } else {
            suit_str = tile.guessed_suit.map_or("", |s| s.symbol());
            value_str = tile.guessed_value.map_or("", |v| v.symbol());
            colour = Color::srgb(0.3, 0.3, 0.3);
        }

        let (mark, mark_colour) = if tile.duplicate {
            (" ✗", Color::srgb(0.8, 0.2, 0.2))
        } else {
            ("", Color::srgb(0.2, 0.2, 0.2))
        };

        commands.spawn((
            Text::new(suit_str),
            TextColor(colour),
            TextFont::from(theme.font.clone()).with_font_size(FontSize::Px(36.0)),
            ChildOf(tile_id),
        ));
        commands.spawn((
            Node {
                position_type: PositionType::Relative,
                top: Val::Px(2.0),
                ..default()
            },
            Text::new(value_str),
            TextColor(colour),
            TextFont::from(theme.font.clone()).with_font_size(FontSize::Px(28.0)),
            ChildOf(tile_id),
        ));

        commands.spawn((
            Node {
                position_type: PositionType::Relative,
                top: Val::Px(2.0),
                ..default()
            },
            Text::new(mark),
            TextColor(mark_colour),
            TextFont::from(theme.symbol_font.clone()).with_font_size(FontSize::Px(28.0)),
            ChildOf(tile_id),
        ));
    }

    for tile in tiles {
        let tile_id = layout_data.get_tile_id(tile.position);

        render_tile(&theme, &mut commands, tile_id, tile);
    }
}

pub fn render_clues(
    theme: Res<Theme>,
    layout_data: Res<LayoutData>,
    clues: Query<&Clue, Changed<Clue>>,
    mut commands: Commands
) {
    for clue in clues {
        let header_id = match clue.location {
            ClueLocation::Top(column) => layout_data.top_ids[column],
            ClueLocation::Left(row) => layout_data.left_ids[row],
            ClueLocation::Right(row) => layout_data.right_ids[row],
            ClueLocation::Bottom(column) => layout_data.bottom_ids[column],
        };

        commands.entity(header_id).despawn_children();

        let (mark, text_colour) = match clue.state {
            ClueGuessState::Incomplete => ("", Color::srgb(0.6, 0.6, 0.6)),
            ClueGuessState::Correct => (" ✓", Color::srgb(0.2, 0.8, 0.2)),
            ClueGuessState::Wrong => (" ✗", Color::srgb(0.8, 0.2, 0.2)),
        };

        let text = format!("{}{}", clue.poker_hand.name(), mark);

        commands.spawn((
            Text::new(text),
            TextColor(text_colour),
            TextFont::from(theme.symbol_font.clone()).with_font_size(FontSize::Px(24.0)),
            TextLayout::justify(Justify::Center),
            ChildOf(header_id),
        ));
    }
}

pub fn render_game_seed(
    theme: Res<Theme>,
    game_seed: Res<GameSeed>,
    node_id: Single<Entity, With<GameSeedLabel>>,
    mut commands: Commands,
) {
    commands.entity(*node_id).despawn_children();

    let text = format!("Seed:\n{}", *game_seed);
    let text_colour = Color::srgb(0.8, 0.8, 0.8);

    commands.spawn((
        Text::new(text),
        TextColor(text_colour),
        TextFont::from(theme.font.clone()).with_font_size(FontSize::Px(24.0)),
        // TextLayout::justify(Justify::Center),
        ChildOf(*node_id),
    ));
}

const MIN_WINDOW_SIZE: Vec2 = Vec2::new(1280.0, 720.0);

pub fn adjust_scaling(
    window: Single<&Window>,
    mut ui_scale: ResMut<UiScale>,
) {
    let window_size = &window.resolution.size();
    let relative_scale = window_size / MIN_WINDOW_SIZE;
    let scale = relative_scale.min_element();
    info!("Window resized to {window_size}; setting UI scale to {scale}");
    ui_scale.0 = scale;
}
