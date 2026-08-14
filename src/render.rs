use bevy::color::Color;
use bevy::prelude::{Changed, ChildOf, Commands, Query, Res, Text};
use bevy::text::{FontSize, Justify, TextColor, TextFont, TextLayout};
use bevy::ui::{BackgroundColor, Node, PositionType, Val};
use bevy::utils::default;

use crate::cards::SuitColour;
use crate::LayoutData;
use crate::game::{Hand, HandGuessState, Tile};

pub(crate) fn render_game(
    data: Res<LayoutData>,
    tiles: Query<&Tile, Changed<Tile>>,
    hands: Query<&Hand, Changed<Hand>>,
    mut commands: Commands
) {
    for i in 0..5 {
        for j in 0..5 {
            let tile_id = data.tile_ids[i][j];

            let Ok(tile) = tiles.get(tile_id)
                else { continue; };

            commands.entity(tile_id).despawn_children();

            let bg = if tile.selected {
                Color::srgb(0.0, 0.0, 0.0)
            } else {
                Color::srgb(0.0, 0.1, 0.0)
            };

            commands.entity(tile_id)
                .insert(BackgroundColor(bg));

            let suit_str;
            let value_str;
            let colour;
            if tile.known {
                suit_str = tile.card.suit.symbol();
                value_str = tile.card.value.symbol();
                colour = match tile.card.suit.colour() {
                    SuitColour::Red => Color::srgb(0.8, 0.1, 0.1),
                    SuitColour::Black => Color::srgb(0.7, 0.7, 0.7),
                };
            } else {
                suit_str = tile.guessed_suit.map_or("", |s| s.symbol());
                value_str = tile.guessed_value.map_or("", |v| v.symbol());
                colour = Color::srgb(0.4, 0.4, 0.4);
            }

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
        }
    }

    for i in 0..5 {
        let column_id = data.column_ids[i];

        let Ok(hand) = hands.get(column_id)
        else { continue; };

        commands.entity(column_id).despawn_children();

        let (mark, text_colour) = match hand.state {
            HandGuessState::Incomplete => ("", Color::srgb(0.6, 0.6, 0.6)),
            HandGuessState::Correct => (" ✓", Color::srgb(0.2, 0.8, 0.2)),
            HandGuessState::Wrong => (" ✗", Color::srgb(0.8, 0.2, 0.2)),
        };

        let text = format!("{}{}", hand.poker_hand.name(), mark);

        commands.spawn((
            Text::new(text),
            TextColor(text_colour),
            TextFont::from(data.symbol_font.clone()).with_font_size(FontSize::Px(24.0)),
            TextLayout::justify(Justify::Center),
            ChildOf(column_id),
        ));
    }

    for i in 0..5 {
        let row_id = data.row_ids[i];

        let Ok(hand) = hands.get(row_id)
        else { continue; };

        commands.entity(row_id).despawn_children();

        let (mark, text_colour) = match hand.state {
            HandGuessState::Incomplete => ("", Color::srgb(0.6, 0.6, 0.6)),
            HandGuessState::Correct => (" ✓", Color::srgb(0.2, 0.8, 0.2)),
            HandGuessState::Wrong => (" ✗", Color::srgb(0.8, 0.2, 0.2)),
        };

        let text = format!("{}{}", hand.poker_hand.name(), mark);

        commands.spawn((
            Text::new(text),
            TextColor(text_colour),
            TextFont::from(data.symbol_font.clone()).with_font_size(FontSize::Px(24.0)),
            TextLayout::justify(Justify::Center),
            ChildOf(row_id),
        ));
    }
}
