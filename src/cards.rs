use rand::Rng;
use rand::seq::SliceRandom;

pub const NUM_SUITS: usize = 4;
pub const CARDS_PER_SUIT: usize = 13;
pub const CARD_VALUES: [u8; CARDS_PER_SUIT] = [2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14];
pub const MIN_CARD_VALUE: u8 = CARD_VALUES[0];
pub const FULL_PACK_SIZE: usize = NUM_SUITS * CARDS_PER_SUIT;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SuitColour {
    Red,
    Black,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

impl Suit {
    pub(crate) fn all() -> [Suit; 4] {
        [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades]
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            Suit::Clubs => "♣",
            Suit::Diamonds => "♦",
            Suit::Hearts => "♥",
            Suit::Spades => "♠",
        }
    }

    pub fn colour(&self) -> SuitColour {
        match self {
            Suit::Clubs => SuitColour::Black,
            Suit::Diamonds => SuitColour::Red,
            Suit::Hearts => SuitColour::Red,
            Suit::Spades => SuitColour::Black,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Value(u8);

impl Value {
    fn all() -> [Value; CARDS_PER_SUIT] {
        CARD_VALUES.map(|i| Value(i))
    }

    pub fn of_number(number: u8) -> Self {
        assert!(number >= 1 && number <= 14);
        Value(number)
    }

    pub fn of_face(face: char) -> Self {
        match face {
            'a' => Value(1),
            'J' => Value(11),
            'Q' => Value(12),
            'K' => Value(13),
            'A' => Value(14),
            _ => unimplemented!(),
        }
    }

    pub fn low_ace() -> Self {
        Value(1)
    }

    pub fn symbol(&self) -> &'static str {
        const SYMBOLS: [&str; 14] = ["a", "2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K", "A"];
        SYMBOLS[self.0 as usize - 1]
    }

    pub fn value(&self) -> u8 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CardId {
    pub suit: Suit,
    pub value: Value,
}

impl CardId {
    pub fn new(suit: Suit, value: Value) -> CardId {
        CardId { suit, value }
    }
}

/** A stack of cards, in a given order. */
#[derive(Debug)]
pub struct Stack {
    size: usize,
    original_size: usize,
    cards: Vec<CardId>,
}

impl Stack {
    pub fn full_pack() -> Self {
        let mut cards = Vec::with_capacity(52);
        for suit in Suit::all() {
            for value in Value::all() {
                cards.push(CardId { suit, value });
            }
        }
        Stack {
            size: cards.len(),
            original_size: cards.len(),
            cards, }
    }

    pub fn shuffle(&mut self, rng: &mut impl Rng) {
        self.cards.shuffle(rng);
    }

    pub fn pop(&mut self) -> Option<CardId> {
        if self.size == 0 { return None; }
        self.size -= 1;
        let card = self.cards[self.size];
        Some(card)
    }

    /**
     * Reset the pack to its original size, restoring any popped cards. Note that if any cards
     * have been pushed to the stack in the interim, they will take the place of popped cards!
     */
    pub fn reset(&mut self) {
        self.size = self.original_size;
    }
}
