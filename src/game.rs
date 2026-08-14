use std::mem::MaybeUninit;

use bevy::log::{info, warn};
use bevy::prelude::{Commands, Component, Entity, In, Query, Res, Resource};
use rand::seq::SliceRandom;

use crate::LayoutData;
use crate::cards::{CardId, Stack, Suit, Value};
use crate::poker::{identify_hand, PokerHand};

#[derive(Default, Resource)]
pub struct Selection {
    pub position: (usize, usize),
}

#[derive(Component)]
pub struct Tile {
    pub position: (usize, usize),
    pub card: CardId,
    pub known: bool,
    pub guessed_suit: Option<Suit>,
    pub guessed_value: Option<Value>,
    pub selected: bool,
}

pub enum HandGuessState {
    Incomplete,
    Correct,
    Wrong,
}

#[derive(Component)]
pub struct Hand {
    pub poker_hand: PokerHand,
    pub state: HandGuessState,
}

pub fn restart_game(
    data: Res<LayoutData>,
    mut commands: Commands,
) {
    info!("Restarting game");

    let (cards, col_hands, row_hands) = make_game();

    let mut missings = (0..5).collect::<Vec<_>>();
    missings.shuffle(&mut rand::rng());


    for i in 0..5 {
        for j in 0..5 {
            let card = cards[i][j];
            let known = missings[i] != j;
            let new_tile = Tile { position: (i, j), card, known, guessed_suit: None, guessed_value: None, selected: false };

            commands.entity(data.tile_ids[i][j])
                .insert(new_tile);
        }
    }

    for j in 0..5 {
        let hand = Hand { poker_hand: col_hands[j], state: HandGuessState::Incomplete };
        commands.entity(data.column_ids[j])
            .insert(hand);
    }

    for i in 0..5 {
        let hand = Hand { poker_hand: row_hands[i], state: HandGuessState::Incomplete };
        commands.entity(data.row_ids[i])
            .insert(hand);
    }
}

fn make_game() -> ([[CardId; 5]; 5], [PokerHand; 5], [PokerHand; 5]) {
    let mut cards: [[MaybeUninit<CardId>; 5]; 5] = [[MaybeUninit::uninit(); 5]; 5];
    let mut row_hands: [MaybeUninit<PokerHand>; 5] = [MaybeUninit::uninit(); 5];
    let mut col_hands: [MaybeUninit<PokerHand>; 5] = [MaybeUninit::uninit(); 5];
    let mut retries = 0;

    loop {
        let mut pack = Stack::full_pack();
        pack.shuffle();

        for i in 0..5 {
            for j in 0..5 {
                let card = pack.pop().expect("nonempty pack");
                cards[i][j].write(card);
            }
        }

        let cards = cards.map(|r| r.map(|c| unsafe { c.assume_init() }));

        let mut hand_counts = [0; 11];

        for j in 0..5 {
            let mut col_cards = (0..5).map(|i| cards[i][j]).collect::<Vec<_>>();
            let poker_hand = identify_hand(&mut col_cards);
            hand_counts[poker_hand as usize] += 1;
            row_hands[j].write(poker_hand);
        }

        for i in 0..5 {
            let mut row_cards = (0..5).map(|j| cards[i][j]).collect::<Vec<_>>();
            let poker_hand = identify_hand(&mut row_cards);
            hand_counts[poker_hand as usize] += 1;
            col_hands[i].write(poker_hand);
        }

        let row_hands = row_hands.map(|h| unsafe { h.assume_init() });
        let col_hands = col_hands.map(|h| unsafe { h.assume_init() });

        if hand_counts[PokerHand::Nothing as usize] >= 2 || hand_counts[PokerHand::OnePair as usize] >= 6
                || hand_counts.iter().filter(|k| **k > 0).count() < 5 {
            retries += 1;
            if retries % 1000 == 0 {
                warn!("Retrying shuffle {retries} times!")
            }

            continue;
        }

        return (cards, row_hands, col_hands);
    }
}

pub fn check_guesses(
    data: Res<LayoutData>,
    tiles: Query<&Tile>,
    mut hands: Query<&mut Hand>,
) {
    for i in 0..5 {
        let mut cards = get_cards_in_column(&data, &tiles, i);
        let guessed_hand = identify_hand(&mut cards);

        let Ok(mut hand) = hands.get_mut(data.column_ids[i])
        else { continue; };

        hand.state = if cards.len() < 5 {
            HandGuessState::Incomplete
        } else if guessed_hand == hand.poker_hand {
            HandGuessState::Correct
        } else {
            HandGuessState::Wrong
        }
    }

    for i in 0..5 {
        let mut cards = get_cards_in_row(&data, &tiles, i);
        let guessed_hand = identify_hand(&mut cards);

        let Ok(mut hand) = hands.get_mut(data.row_ids[i])
        else { continue; };

        hand.state = if cards.len() < 5 {
            HandGuessState::Incomplete
        } else if guessed_hand == hand.poker_hand {
            HandGuessState::Correct
        } else {
            HandGuessState::Wrong
        }
    }
}

pub fn select_tile(
    mut tiles: Query<&mut Tile>,
    selection: Res<Selection>,
) {
    info!("Selecting tile");

    for mut t in tiles.iter_mut() {
        let should_select = t.position == selection.position;

        if t.selected != should_select {
            t.selected = should_select;
        }
    }
}

pub fn guess_suit(
    suit: In<Suit>,
    mut tiles: Query<&mut Tile>,
    layout_data: Res<LayoutData>,
    selection: Res<Selection>,
) {
    info!("Guessing suit: {}", suit.symbol());

    let tile_id = layout_data.tile_ids[selection.position.0][selection.position.1];
    let Ok(mut tile) = tiles.get_mut(tile_id)
    else { return; };

    if tile.known { return; }

    tile.guessed_suit = Some(*suit);
}

pub fn guess_value(
    value: In<Value>,
    mut tiles: Query<&mut Tile>,
    layout_data: Res<LayoutData>,
    selection: Res<Selection>,
) {
    info!("Guessing value: {}", value.symbol());

    let tile_id = layout_data.tile_ids[selection.position.0][selection.position.1];
    let Ok(mut tile) = tiles.get_mut(tile_id)
    else { return; };

    if tile.known { return; }

    tile.guessed_value = Some(*value);
}

pub fn clear_guesses(
    mut tiles: Query<&mut Tile>,
    layout_data: Res<LayoutData>,
    selection: Res<Selection>,
) {
    info!("Clearing guesses");

    let tile_id = layout_data.tile_ids[selection.position.0][selection.position.1];
    let Ok(mut tile) = tiles.get_mut(tile_id)
    else { return; };

    if tile.known { return; }

    tile.guessed_suit = None;
    tile.guessed_value = None;
}

fn get_cards_in_column(layout_data: &LayoutData, tiles: &Query<&Tile>, column: usize) -> Vec<CardId> {
    let mut cards = Vec::new();
    for row in 0..5 {
        let tile_id = layout_data.tile_ids[row][column];
        let Ok(tile) = tiles.get(tile_id)
        else { continue; };
        let card = if tile.known {
            tile.card
        } else if let (Some(suit), Some(value)) = (tile.guessed_suit, tile.guessed_value) {
            CardId::new(suit, value)
        } else {
            continue;
        };
        cards.push(card)
    }
    cards
}

fn get_cards_in_row(layout_data: &LayoutData, tiles: &Query<&Tile>, row: usize) -> Vec<CardId> {
    let mut cards = Vec::new();
    for column in 0..5 {
        let tile_id = layout_data.tile_ids[row][column];
        let Ok(tile) = tiles.get(tile_id)
        else { continue; };
        let card = if tile.known {
            tile.card
        } else if let (Some(suit), Some(value)) = (tile.guessed_suit, tile.guessed_value) {
            CardId::new(suit, value)
        } else {
            continue;
        };
        cards.push(card)
    }
    cards
}
