use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use bevy::asset::Handle;
use bevy::ecs::{
    component::Component,
    entity::Entity,
    message::Message,
    resource::Resource,
    system::{Commands, In, Query, Res, ResMut},
};
use bevy::log::info;
use bevy::text::Font;
use bevy::utils::default;
use rand::{Rng, RngExt, SeedableRng};
use rand::seq::SliceRandom;

use crate::cards::{CardId, Suit, Value, FULL_PACK_SIZE};
use crate::deal::deal_game;
use crate::poker::{identify_hand, PokerHand, POKER_HAND_INDICES, POKER_HAND_SIZE};

pub const CLUES_PER_DIRECTION: usize = POKER_HAND_SIZE;
pub const CLUE_INDICES: [usize; CLUES_PER_DIRECTION] = POKER_HAND_INDICES;
pub const NUM_PLANES: usize = 2;
pub const PLANE_INDICES: [usize; NUM_PLANES] = [0, 1];
pub const NUM_SPARES: usize = FULL_PACK_SIZE - CLUES_PER_DIRECTION * CLUES_PER_DIRECTION * NUM_PLANES;
pub const SPARE_INDICES: [usize; NUM_SPARES] = [0, 1];

#[derive(Clone, Copy, Debug, Resource)]
pub struct GameSeed(u16);

impl GameSeed {
    pub fn rng(&self) -> impl Rng {
        rand::rngs::Xoshiro128PlusPlus::seed_from_u64(self.0 as u64)
    }
}

impl Default for GameSeed {
    fn default() -> Self {
        Self(rand::rng().random())
    }
}

impl Display for GameSeed {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04X}", self.0)
    }
}

#[derive(Debug, Message)]
pub enum GameMessage {
    Redeal,
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
    pub(crate) fn tile_positions(&self) -> [(usize, usize, usize); POKER_HAND_SIZE] {
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

pub fn redeal_game(
    data: Res<LayoutData>,
    mut seed: ResMut<GameSeed>,
    mut commands: Commands,
) {
    *seed = default();
    info!("Dealing game with seed {:?}", seed);

    let dealt = deal_game(*seed);

    let mut missings = CLUE_INDICES;
    missings.shuffle(&mut seed.rng());

    for i in CLUE_INDICES {
        for j in CLUE_INDICES {
            for k in PLANE_INDICES {
                let card = dealt.board[i][j][k];
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

    build_clues(&mut commands, ClueLocation::Top, &dealt.top_hands, &data.top_ids);
    build_clues(&mut commands, ClueLocation::Left, &dealt.left_hands, &data.left_ids);
    build_clues(&mut commands, ClueLocation::Right, &dealt.right_hands, &data.right_ids);
    build_clues(&mut commands, ClueLocation::Bottom, &dealt.bottom_hands, &data.bottom_ids);

    for (index, card) in dealt.spares.iter().enumerate() {
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
