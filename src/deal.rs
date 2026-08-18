use std::mem::MaybeUninit;

use bevy::log::warn;

use crate::cards::{CardId, Stack};
use crate::game::{ClueLocation, CLUES_PER_DIRECTION, CLUE_INDICES, NUM_PLANES, NUM_SPARES, PLANE_INDICES, SPARE_INDICES};
use crate::poker::{identify_hand, PokerHand, NUM_HAND_TYPES};

pub struct DealtGame {
    pub board: [[[CardId; NUM_PLANES]; CLUES_PER_DIRECTION]; CLUES_PER_DIRECTION],
    pub spares: [CardId; NUM_SPARES],
    pub top_hands: [PokerHand; CLUES_PER_DIRECTION],
    pub left_hands: [PokerHand; CLUES_PER_DIRECTION],
    pub right_hands: [PokerHand; CLUES_PER_DIRECTION],
    pub bottom_hands: [PokerHand; CLUES_PER_DIRECTION],
}

pub fn deal_game() -> DealtGame {
    let mut board = [[[MaybeUninit::uninit(); NUM_PLANES]; CLUES_PER_DIRECTION]; CLUES_PER_DIRECTION];

    let mut retries = 0;

    loop {
        let mut pack = Stack::full_pack();
        pack.shuffle();

        for i in CLUE_INDICES {
            for j in CLUE_INDICES {
                for k in PLANE_INDICES {
                    let card = pack.pop().expect("nonempty pack");
                    board[i][j][k].write(card);
                }
            }
        }

        let board = board.map(
            |r| r.map(
                |c| c.map(
                    |p| unsafe { p.assume_init() }
                )));

        fn build_hands(board: &[[[CardId; NUM_PLANES]; CLUES_PER_DIRECTION]; CLUES_PER_DIRECTION], constr: fn(usize) -> ClueLocation, hand_counts: &mut[i32]) -> [PokerHand; CLUES_PER_DIRECTION] {
            let mut hands: [MaybeUninit<PokerHand>; CLUES_PER_DIRECTION] = [MaybeUninit::uninit(); CLUES_PER_DIRECTION];
            for i in CLUE_INDICES {
                let loc = constr(i);
                let mut cards = loc.tile_positions().map(|(i, j, k)| board[i][j][k]);
                let poker_hand = identify_hand(&mut cards);
                hand_counts[poker_hand as usize] += 1;
                hands[i].write(poker_hand);
            }

            hands.map(|h| unsafe { h.assume_init() })
        }

        let mut hand_counts = [0; NUM_HAND_TYPES];

        let top_hands = build_hands(&board, ClueLocation::Top, &mut hand_counts);
        let left_hands = build_hands(&board, ClueLocation::Left, &mut hand_counts);
        let right_hands = build_hands(&board, ClueLocation::Right, &mut hand_counts);
        let bottom_hands = build_hands(&board, ClueLocation::Bottom, &mut hand_counts);

        // Check for sufficient interesting hands; redeal if not good enough
        if hand_counts[PokerHand::Nothing as usize] >= 6 || hand_counts[PokerHand::OnePair as usize] >= 12
            || hand_counts.iter().filter(|k| **k > 0).count() < 4 {
            retries += 1;
            if retries % 1000 == 0 {
                warn!("Retrying shuffle {retries} times!")
            }

            continue;
        }

        let spares = SPARE_INDICES.map(|_| pack.pop().expect("nonempty pack"));

        return DealtGame {
            board,
            spares,
            top_hands,
            left_hands,
            right_hands,
            bottom_hands,
        }
    }
}
