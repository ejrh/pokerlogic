use std::collections::HashMap;
use std::mem::MaybeUninit;

use bevy::asset::Handle;
use bevy::ecs::{
    component::Component,
    entity::Entity,
    message::Message,
    resource::Resource,
    system::{Commands, In, Query, Res},
};
use bevy::log::{info, warn};
use bevy::text::Font;
use rand::seq::SliceRandom;

use crate::cards::{CardId, Stack, Suit, Value, FULL_PACK_SIZE};
use crate::poker::{identify_hand, PokerHand, NUM_HAND_TYPES, POKER_HAND_INDICES, POKER_HAND_SIZE};

pub const CLUES_PER_DIRECTION: usize = POKER_HAND_SIZE;
pub const CLUE_INDICES: [usize; CLUES_PER_DIRECTION] = POKER_HAND_INDICES;
pub const NUM_PLANES: usize = 2;
pub const PLANE_INDICES: [usize; NUM_PLANES] = [0, 1];
pub const NUM_SPARES: usize = FULL_PACK_SIZE - CLUES_PER_DIRECTION * CLUES_PER_DIRECTION * NUM_PLANES;

#[derive(Debug, Message)]
pub enum GameMessage {
    Restart,
    Quit,
    SelectTile,
    GuessSuit(Suit),
    GuessValue(Value),
    ClearGuesses,
    SolveAll,
    Victory,
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum TilePosition {
    Board(usize, usize, usize),
    Spare(usize),
}

#[derive(Default, Resource)]
pub struct Selection {
    pub position: Option<TilePosition>,
}

#[derive(Component)]
pub struct Tile {
    pub position: TilePosition,
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
    fn tile_positions(&self) -> [(usize, usize, usize); POKER_HAND_SIZE] {
        match self {
            ClueLocation::Top(column) => CLUE_INDICES.map(|row| (row, *column, 0)),
            ClueLocation::Left(row) => CLUE_INDICES.map(|column| (*row, column, 0)),
            ClueLocation::Right(row) => CLUE_INDICES.map(|column| (*row, column, 1)),
            ClueLocation::Bottom(column) => CLUE_INDICES.map(|row| (row, *column, 1)),
        }
    }
}

#[derive(Component)]
pub struct Clue {
    pub poker_hand: PokerHand,
    pub state: ClueGuessState,
    pub location: ClueLocation,
}

#[derive(Resource)]
pub struct LayoutData {
    pub font: Handle<Font>,
    pub symbol_font: Handle<Font>,
    pub top_ids: Vec<Entity>,
    pub left_ids: Vec<Entity>,
    pub right_ids: Vec<Entity>,
    pub bottom_ids: Vec<Entity>,
    pub tile_ids: Vec<Vec<[Entity; 2]>>,
    pub spare_ids: Vec<Entity>,
}

impl LayoutData {
    pub fn get_tile_id(&self, position: TilePosition) -> Entity {
        match position {
            TilePosition::Board(row, column, plane) => self.tile_ids[row][column][plane],
            TilePosition::Spare(index) => self.spare_ids[index],
        }
    }
}

impl Default for LayoutData {
    fn default() -> Self {
        LayoutData {
            font: Handle::default(),
            symbol_font: Handle::default(),
            top_ids: vec![Entity::PLACEHOLDER; CLUES_PER_DIRECTION],
            left_ids: vec![Entity::PLACEHOLDER; CLUES_PER_DIRECTION],
            right_ids: vec![Entity::PLACEHOLDER; CLUES_PER_DIRECTION],
            bottom_ids: vec![Entity::PLACEHOLDER; CLUES_PER_DIRECTION],
            tile_ids: vec![vec![[Entity::PLACEHOLDER; NUM_PLANES]; CLUES_PER_DIRECTION]; CLUES_PER_DIRECTION],
            spare_ids: vec![Entity::PLACEHOLDER; NUM_SPARES],
        }
    }
}

pub fn restart_game(
    data: Res<LayoutData>,
    mut commands: Commands,
) {
    info!("Restarting game");

    let (cards, spares, top_hands, left_hands, right_hands, bottom_hands) = make_game();

    let mut missings = CLUE_INDICES.clone();
    missings.shuffle(&mut rand::rng());

    for i in CLUE_INDICES {
        for j in CLUE_INDICES {
            for k in PLANE_INDICES {
                let card = cards[i][j][k];
                let known = missings[i] != j;
                let new_tile = Tile {
                    position: TilePosition::Board(i, j, k),
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

    fn build_clues(commands: &mut Commands, constr: fn(usize) -> ClueLocation, hands: &[PokerHand], ids: &[Entity]) {
        for i in CLUE_INDICES {
            let clue = Clue {
                poker_hand: hands[i],
                state: ClueGuessState::Incomplete,
                location: constr(i)
            };
            commands.entity(ids[i])
                .insert(clue);
        }
    }

    build_clues(&mut commands, ClueLocation::Top, &top_hands, &data.top_ids);
    build_clues(&mut commands, ClueLocation::Left, &left_hands, &data.left_ids);
    build_clues(&mut commands, ClueLocation::Right, &right_hands, &data.right_ids);
    build_clues(&mut commands, ClueLocation::Bottom, &bottom_hands, &data.bottom_ids);

    for (index, card) in spares.iter().enumerate() {
        let new_tile = Tile {
            position: TilePosition::Spare(index),
            card: *card,
            known: true,
            guessed_suit: None,
            guessed_value: None,
            selected: false,
            duplicate: false,
        };

        commands.entity(data.spare_ids[index])
            .insert(new_tile);
    }

    commands.insert_resource(Selection::default());
}

fn make_game() -> ([[[CardId; NUM_PLANES]; CLUES_PER_DIRECTION]; CLUES_PER_DIRECTION], Vec<CardId>, [PokerHand; CLUES_PER_DIRECTION], [PokerHand; CLUES_PER_DIRECTION], [PokerHand; CLUES_PER_DIRECTION], [PokerHand; CLUES_PER_DIRECTION]) {
    let mut cards: [[[MaybeUninit<CardId>; NUM_PLANES]; CLUES_PER_DIRECTION]; CLUES_PER_DIRECTION] = [[[MaybeUninit::uninit(); NUM_PLANES]; CLUES_PER_DIRECTION]; CLUES_PER_DIRECTION];

    let mut retries = 0;

    loop {
        let mut pack = Stack::full_pack();
        pack.shuffle();

        for i in CLUE_INDICES {
            for j in CLUE_INDICES {
                for k in PLANE_INDICES {
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

        fn build_hands(cards: &[[[CardId; NUM_PLANES]; CLUES_PER_DIRECTION]; CLUES_PER_DIRECTION], constr: fn(usize) -> ClueLocation, hand_counts: &mut[i32]) -> [PokerHand; CLUES_PER_DIRECTION] {
            let mut hands: [MaybeUninit<PokerHand>; CLUES_PER_DIRECTION] = [MaybeUninit::uninit(); CLUES_PER_DIRECTION];
            for i in CLUE_INDICES {
                let loc = constr(i);
                let mut cards = loc.tile_positions().map(|(i, j, k)| cards[i][j][k]);
                let poker_hand = identify_hand(&mut cards);
                hand_counts[poker_hand as usize] += 1;
                hands[i].write(poker_hand);
            }

            hands.map(|h| unsafe { h.assume_init() })
        }

        let mut hand_counts = [0; NUM_HAND_TYPES];

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

        let spares = pack.pop_all();
        return (cards, spares, top_hands, left_hands, right_hands, bottom_hands);
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

        clue.state = if cards.len() < POKER_HAND_SIZE {
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
        let should_select = matches!(selection.position, Some(pos) if pos == t.position);

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

    let Some(mut tile) = selection.position
        .map(|p| layout_data.get_tile_id(p))
        .and_then(|id| tiles.get_mut(id).ok())
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

    let Some(mut tile) = selection.position
        .map(|p| layout_data.get_tile_id(p))
        .and_then(|id| tiles.get_mut(id).ok())
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

    let Some(mut tile) = selection.position
        .map(|p| layout_data.get_tile_id(p))
        .and_then(|id| tiles.get_mut(id).ok())
    else { return; };

    if tile.known { return; }

    tile.guessed_suit = None;
    tile.guessed_value = None;
}

pub fn solve_all(
    tiles: Query<&mut Tile>,
) {
    for mut tile in tiles {
        tile.guessed_suit = Some(tile.card.suit);
        tile.guessed_value = Some(tile.card.value);
    }
}

fn get_cards_for_clue(layout_data: &LayoutData, tiles: Query<&Tile>, location: ClueLocation) -> Vec<CardId> {
    let mut cards = Vec::new();
    for (row, column, plane) in location.tile_positions() {
        let tile_id = layout_data.get_tile_id(TilePosition::Board(row, column, plane));

        let Ok(tile) = tiles.get(tile_id)
        else { continue; };

        let Some(card) = tile.effective_card()
        else { continue; };
        cards.push(card)
    }
    cards
}
