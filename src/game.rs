use std::collections::HashMap;
use std::mem::MaybeUninit;

use bevy::log::{info, warn};
use bevy::prelude::{Commands, Component, Entity, In, Query, Res, Resource};
use rand::seq::SliceRandom;

use crate::{GameMessage, LayoutData};
use crate::cards::{CardId, Stack, Suit, Value};
use crate::poker::{identify_hand, PokerHand, HAND_INDICES, HAND_SIZE};

#[derive(Default, Resource)]
pub struct Selection {
    pub position: (usize, usize, usize),
}

#[derive(Component)]
pub struct Tile {
    pub position: (usize, usize, usize),
    pub card: CardId,
    pub known: bool,
    pub guessed_suit: Option<Suit>,
    pub guessed_value: Option<Value>,
    pub selected: bool,
    pub duplicate: bool,
}

impl Tile {
    pub fn effective_card(&self) -> Option<CardId> {
        if self.known {
            Some(self.card)
        } else if let (Some(suit), Some(value)) = (self.guessed_suit, self.guessed_value) {
            Some(CardId::new(suit, value))
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClueGuessState {
    Incomplete,
    Correct,
    Wrong,
}

#[derive(Clone, Copy, Debug)]
pub enum ClueLocation {
    Top(usize),
    Left(usize),
    Right(usize),
    Bottom(usize),
}

impl ClueLocation {
    fn tile_positions(&self) -> [(usize, usize, usize); HAND_SIZE] {
        match self {
            ClueLocation::Top(column) => HAND_INDICES.map(|row| (row, *column, 0)),
            ClueLocation::Left(row) => HAND_INDICES.map(|column| (*row, column, 0)),
            ClueLocation::Right(row) => HAND_INDICES.map(|column| (*row, column, 1)),
            ClueLocation::Bottom(column) => HAND_INDICES.map(|row| (row, *column, 1)),
        }
    }
}

#[derive(Component)]
pub struct Clue {
    pub poker_hand: PokerHand,
    pub state: ClueGuessState,
    pub location: ClueLocation,
}

pub fn restart_game(
    data: Res<LayoutData>,
    mut commands: Commands,
) {
    info!("Restarting game");

    let (cards, top_hands, left_hands, right_hands, bottom_hands) = make_game();

    let mut missings = (0..5).collect::<Vec<_>>();
    missings.shuffle(&mut rand::rng());

    for i in 0..5 {
        for j in 0..5 {
            for k in 0..2 {
                let card = cards[i][j][k];
                let known = missings[i] != j;
                let new_tile = Tile {
                    position: (i, j, k),
                    card,
                    known,
                    guessed_suit: None,
                    guessed_value: None,
                    selected: false,
                    duplicate: false,
                };

                commands.entity(data.tile_ids[i][j][k])
                    .insert(new_tile);
            }
        }
    }

    fn build_clue(commands: &mut Commands, constr: fn(usize) -> ClueLocation, hands: &[PokerHand], ids: &[Entity]) {
        for i in 0..5 {
            let clue = Clue {
                poker_hand: hands[i],
                state: ClueGuessState::Incomplete,
                location: constr(i)
            };
            commands.entity(ids[i])
                .insert(clue);
        }
    }

    build_clue(&mut commands, ClueLocation::Top, &top_hands, &data.top_ids);
    build_clue(&mut commands, ClueLocation::Left, &left_hands, &data.left_ids);
    build_clue(&mut commands, ClueLocation::Right, &right_hands, &data.right_ids);
    build_clue(&mut commands, ClueLocation::Bottom, &bottom_hands, &data.bottom_ids);
}

fn make_game() -> ([[[CardId; 2]; 5]; 5], [PokerHand; 5], [PokerHand; 5], [PokerHand; 5], [PokerHand; 5]) {
    let mut cards: [[[MaybeUninit<CardId>; 2]; 5]; 5] = [[[MaybeUninit::uninit(); 2]; 5]; 5];

    let mut retries = 0;

    loop {
        let mut pack = Stack::full_pack();
        pack.shuffle();

        for i in 0..5 {
            for j in 0..5 {
                for k in 0..2 {
                    let card = pack.pop().expect("nonempty pack");
                    cards[i][j][k].write(card);
                }
            }
        }

        let cards = cards.map(
            |r| r.map(
                |c| c.map(
                    |p| unsafe { p.assume_init() }
        )));

        fn build_hands(cards: &[[[CardId; 2]; 5]; 5], constr: fn(usize) -> ClueLocation, hand_counts: &mut[i32]) -> [PokerHand; 5] {
            let mut hands: [MaybeUninit<PokerHand>; 5] = [MaybeUninit::uninit(); 5];
            for i in 0..5 {
                let loc = constr(i);
                let mut cards = loc.tile_positions().map(|(i, j, k)| cards[i][j][k]);
                let poker_hand = identify_hand(&mut cards);
                hand_counts[poker_hand as usize] += 1;
                hands[i].write(poker_hand);
            }

            hands.map(|h| unsafe { h.assume_init() })
        }

        let mut hand_counts = [0; 11];

        let top_hands = build_hands(&cards, ClueLocation::Top, &mut hand_counts);
        let left_hands = build_hands(&cards, ClueLocation::Left, &mut hand_counts);
        let right_hands = build_hands(&cards, ClueLocation::Right, &mut hand_counts);
        let bottom_hands = build_hands(&cards, ClueLocation::Bottom, &mut hand_counts);

        // Check for sufficient interesting hands; redeal if not good enough
        if hand_counts[PokerHand::Nothing as usize] >= 6 || hand_counts[PokerHand::OnePair as usize] >= 12
                || hand_counts.iter().filter(|k| **k > 0).count() < 4 {
            retries += 1;
            if retries % 1000 == 0 {
                warn!("Retrying shuffle {retries} times!")
            }

            continue;
        }

        return (cards, top_hands, left_hands, right_hands, bottom_hands);
    }
}

pub fn check_guesses(
    data: Res<LayoutData>,
    tiles: Query<&mut Tile>,
    clues: Query<&mut Clue>,
) {
    for mut clue in clues {
        let mut cards = get_cards_for_clue(&data, tiles.as_readonly(), clue.location);
        let guessed_hand = identify_hand(&mut cards);

        clue.state = if cards.len() < 5 {
            ClueGuessState::Incomplete
        } else if guessed_hand == clue.poker_hand {
            ClueGuessState::Correct
        } else {
            ClueGuessState::Wrong
        }
    }

    let mut tile_cards: HashMap<CardId, u32> = HashMap::new();
    for tile in tiles.iter() {
        let Some(card) = tile.effective_card()
        else { continue; };

        *tile_cards.entry(card).or_default() += 1;
    }

    for mut tile in tiles {
        let duplicate = tile.effective_card().map(
            |card| *tile_cards.entry(card).or_default() > 1
        ).unwrap_or(false);

        if tile.duplicate != duplicate {
            tile.duplicate = duplicate;
        }
    }
}

pub fn check_for_victory(
    clues: Query<&Clue>,
    tiles: Query<&Tile>,
    mut commands: Commands,
) {
    let all_correct = clues.iter().all(|clue| clue.state == ClueGuessState::Correct)
        && tiles.iter().all(|tile| !tile.duplicate);
    if all_correct {
        commands.write_message(GameMessage::Victory);
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

    let tile_id = layout_data.tile_ids[selection.position.0][selection.position.1][selection.position.2];
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

    let tile_id = layout_data.tile_ids[selection.position.0][selection.position.1][selection.position.2];
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

    let tile_id = layout_data.tile_ids[selection.position.0][selection.position.1][selection.position.2];
    let Ok(mut tile) = tiles.get_mut(tile_id)
    else { return; };

    if tile.known { return; }

    tile.guessed_suit = None;
    tile.guessed_value = None;
}

fn get_cards_for_clue(layout_data: &LayoutData, tiles: Query<&Tile>, location: ClueLocation) -> Vec<CardId> {
    let mut cards = Vec::new();
    for (row, column, plane) in location.tile_positions() {
        let tile_id = layout_data.tile_ids[row][column][plane];

        let Ok(tile) = tiles.get(tile_id)
        else { continue; };

        let Some(card) = tile.effective_card()
        else { continue; };
        cards.push(card)
    }
    cards
}
