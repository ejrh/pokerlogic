use crate::cards::{CardId, Value, CARD_VALUES, MIN_CARD_VALUE};

pub const POKER_HAND_SIZE: usize = 5;
pub const POKER_HAND_INDICES: [usize; POKER_HAND_SIZE] = [0, 1, 2, 3, 4];
pub const NUM_HAND_TYPES: usize = 11;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd)]
pub enum PokerHand {
    Nothing,
    OnePair,
    TwoPair,
    ThreeOfAKind,
    Straight,
    Flush,
    FullHouse,
    FourOfAKind,
    StraightFlush,
    RoyalFlush,
    FiveOfAKind,
}

impl PokerHand {
    pub fn name(&self) -> &'static str {
        match self {
            PokerHand::Nothing => "Nothing",
            PokerHand::OnePair => "One Pair",
            PokerHand::TwoPair => "Two Pair",
            PokerHand::ThreeOfAKind => "Three Of A Kind",
            PokerHand::Straight => "Straight",
            PokerHand::Flush => "Flush",
            PokerHand::FullHouse => "Full House",
            PokerHand::FourOfAKind => "Four Of A Kind",
            PokerHand::StraightFlush => "Straight Flush",
            PokerHand::RoyalFlush => "Royal Flush",
            PokerHand::FiveOfAKind => "Five Of A Kind",
        }
    }
}

pub fn identify_hand(cards: &mut [CardId]) -> PokerHand {
    if cards.is_empty() {
        return PokerHand::Nothing;
    }

    let mut frequencies = CARD_VALUES.map(|v| (v, 0));

    for card in cards.iter() {
        frequencies[(card.value.value() - MIN_CARD_VALUE) as usize].1 += 1;
    }

    frequencies.sort_by_key(|(_c, k)| -k);

    let one_suit_only = cards[1..].iter().all(|c| c.suit == cards[0].suit);

    cards.sort_by_key(|c| c.value);

    // If they have a 2, then Ace is treated as value 1
    if cards[0].value.value() == 2 {
        for c in cards.iter_mut() {
            if c.value.value() == 14 {
                c.value = Value::low_ace();
            }
        }
        cards.sort_by_key(|c| c.value);
    }

    let has_run = cards.windows(2).all(|w| w[0].value.value() + 1 == w[1].value.value());
    let lowest = cards[0].value.value();

    if frequencies[0].1 >= 5 {
        return PokerHand::FiveOfAKind;
    }

    if has_run && one_suit_only && lowest == 10 {
        return PokerHand::RoyalFlush;
    }

    if has_run && one_suit_only {
        return PokerHand::StraightFlush;
    }

    if frequencies[0].1 >= 4 {
        return PokerHand::FourOfAKind;
    }

    if frequencies[0].1 >= 3 && frequencies[1].1 >= 2 {
        return PokerHand::FullHouse;
    }

    if one_suit_only {
        return PokerHand::Flush;
    }

    if has_run {
        return PokerHand::Straight;
    }

    if frequencies[0].1 >= 3 {
        return PokerHand::ThreeOfAKind;
    }

    if frequencies[0].1 >= 2 && frequencies[1].1 >= 2 {
        return PokerHand::TwoPair;
    }

    if frequencies[0].1 >= 2 {
        return PokerHand::OnePair;
    }

    PokerHand::Nothing
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::cards::Suit;

    #[test]
    fn straight() {
        let mut pack = [
            CardId::new(Suit::Hearts, Value::of_number(3)),
            CardId::new(Suit::Hearts, Value::of_number(7)),
            CardId::new(Suit::Clubs, Value::of_number(5)),
            CardId::new(Suit::Spades, Value::of_number(6)),
            CardId::new(Suit::Diamonds, Value::of_number(4)),
        ];

        let hand = identify_hand(&mut pack);
        assert_eq!(hand, PokerHand::Straight);
    }

    #[test]
    fn straight_flush() {
        let mut pack = [
            CardId::new(Suit::Clubs, Value::of_number(3)),
            CardId::new(Suit::Clubs, Value::of_number(7)),
            CardId::new(Suit::Clubs, Value::of_number(5)),
            CardId::new(Suit::Clubs, Value::of_number(6)),
            CardId::new(Suit::Clubs, Value::of_number(4)),
        ];

        let hand = identify_hand(&mut pack);
        assert_eq!(hand, PokerHand::StraightFlush);
    }

    #[test]
    fn straight_flush_with_ace() {
        let mut pack = [
            CardId::new(Suit::Clubs, Value::of_number(3)),
            CardId::new(Suit::Clubs, Value::of_number(2)),
            CardId::new(Suit::Clubs, Value::of_number(5)),
            CardId::new(Suit::Clubs, Value::of_face('A')),
            CardId::new(Suit::Clubs, Value::of_number(4)),
        ];

        let hand = identify_hand(&mut pack);
        assert_eq!(hand, PokerHand::StraightFlush);
    }

    #[test]
    fn royal_flush() {
        let mut pack = [
            CardId::new(Suit::Clubs, Value::of_face('Q')),
            CardId::new(Suit::Clubs, Value::of_number(10)),
            CardId::new(Suit::Clubs, Value::of_face('A')),
            CardId::new(Suit::Clubs, Value::of_face('K')),
            CardId::new(Suit::Clubs, Value::of_face('J')),
        ];

        let hand = identify_hand(&mut pack);
        assert_eq!(hand, PokerHand::RoyalFlush);
    }
}
