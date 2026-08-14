use bevy::color::Color;
use bevy::prelude::{BorderColor, Changed, ChildOf, Commands, Query, Res, Text};
use bevy::text::{FontSize, Justify, TextColor, TextFont, TextLayout};
use bevy::ui::{BackgroundColor, Node, PositionType, Val};
use bevy::utils::default;

use crate::cards::SuitColour;
use crate::LayoutData;
use crate::game::{Clue, ClueGuessState, ClueLocation, Tile};

pub fn render_tiles(
    data: Res<LayoutData>,
    tiles: Query<&Tile, Changed<Tile>>,
    mut commands: Commands
) {
    for i in 0..5 {
        for j in 0..5 {
            let tile_id = data.tile_ids[i][j];

            let Ok(tile) = tiles.get(tile_id)
                else { continue; };

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
                Color::srgb(0.1, 0.1, 0.11)
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
    }
}

pub fn render_clues(
    data: Res<LayoutData>,
    clues: Query<&Clue, Changed<Clue>>,
    mut commands: Commands
) {
    for clue in clues {
        let header_id = match clue.location {
            ClueLocation::Column(column) => data.column_ids[column],
            ClueLocation::Row(row) => data.row_ids[row],
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

