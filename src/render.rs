use bevy::color::Color;
use bevy::ecs::{
    entity::Entity,
    hierarchy::ChildOf,
    query::Changed,
    system::{Commands, Query, Res},
};
use bevy::text::{FontSize, Justify, TextColor, TextFont, TextLayout};
use bevy::ui::{widget::Text, BackgroundColor, BorderColor, Node, PositionType, Val};
use bevy::utils::default;

use crate::cards::SuitColour;
use crate::game::{Clue, ClueGuessState, ClueLocation, Tile};
use crate::LayoutData;

pub fn render_tiles(
    data: Res<LayoutData>,
    tiles: Query<&Tile, Changed<Tile>>,
    mut commands: Commands
) {
    fn render_tile(data: &LayoutData, commands: &mut Commands, tile_id: Entity, tile: &Tile) {
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
            TextFont::from(data.font.clone()).with_font_size(FontSize::Px(36.0)),
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
            TextFont::from(data.font.clone()).with_font_size(FontSize::Px(28.0)),
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
            TextFont::from(data.symbol_font.clone()).with_font_size(FontSize::Px(28.0)),
            ChildOf(tile_id),
        ));
    }

    for tile in tiles {
        let tile_id = data.get_tile_id(tile.position);

        render_tile(&data, &mut commands, tile_id, tile);
    }
}

pub fn render_clues(
    data: Res<LayoutData>,
    clues: Query<&Clue, Changed<Clue>>,
    mut commands: Commands
) {
    for clue in clues {
        let header_id = match clue.location {
            ClueLocation::Top(column) => data.top_ids[column],
            ClueLocation::Left(row) => data.left_ids[row],
            ClueLocation::Right(row) => data.right_ids[row],
            ClueLocation::Bottom(column) => data.bottom_ids[column],
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
            TextFont::from(data.symbol_font.clone()).with_font_size(FontSize::Px(24.0)),
            TextLayout::justify(Justify::Center),
            ChildOf(header_id),
        ));
    }
}

