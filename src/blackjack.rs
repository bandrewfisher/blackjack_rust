use macroquad::rand::{gen_range, srand};
use macroquad::miniquad::date;

#[derive(Debug, Clone, Copy)]
pub enum Suit {
    Spades,
    Clubs,
    Diamonds,
    Hearts,
}

#[derive(Debug, Clone, Copy)]
pub enum Rank {
    Number(u32),
    Jack,
    Queen,
    King,
    Ace,
}

#[derive(Debug, Clone, Copy)]
pub struct Card {
    pub suit: Suit,
    pub rank: Rank,
}

impl Card {
    /*
    Creates a new card from a string like "AS", "10H"
    */
    pub fn from_repr(card_repr: &str) -> Self {
        let (rank_str, suit_str) = card_repr.split_at(card_repr.len() - 1);

        let rank = match rank_str {
            "J" => Rank::Jack,
            "Q" => Rank::Queen,
            "K" => Rank::King,
            "A" => Rank::Ace,
            _ => {
                let n = rank_str.parse::<u32>().expect("Expected rank to be a number");
                assert!((2..=10).contains(&n), "Invalid number for card rank");
                Rank::Number(n)
            }
        };

        let suit = match suit_str {
            "H" => Suit::Hearts,
            "C" => Suit::Clubs,
            "D" => Suit::Diamonds,
            "S" => Suit::Spades,
            _ => {
                panic!("Invalid suit: {}", &suit_str);
            }
        };

        Self {
            rank,
            suit
        }
    }

    pub fn repr(&self) -> String {
        let mut card_str = String::new();

        match self.rank {
            Rank::Number(num) => {
                card_str.push_str(&num.to_string());
            }
            Rank::Jack => {
                card_str.push_str("J");
            }
            Rank::Queen => {
                card_str.push_str("Q");
            }
            Rank::King => {
                card_str.push_str("K");
            }
            Rank::Ace => {
                card_str.push_str("A");
            }
        }

        match self.suit {
            Suit::Spades => {
                // card_str.push('\u{2660}');
                card_str.push('S');
            }
            Suit::Hearts => {
                // card_str.push('\u{2665}');
                card_str.push('H');
            }
            Suit::Diamonds => {
                // card_str.push('\u{2666}');
                card_str.push('D');
            }
            Suit::Clubs => {
                // card_str.push('\u{2663}');
                card_str.push('C');
            }
        }

        card_str
    }
}

fn shuffle<T>(vec: &mut Vec<T>) {
    let seed = date::now() as u64;
    srand(seed);

    let len = vec.len();

    for i in (1..len).rev() {
        let j = gen_range(0, i);
        vec.swap(i, j);
    }
}

struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    fn new_shuffled() -> Self {
        let mut deck: Vec<Card> = Vec::new();

        for &suit in &[Suit::Spades, Suit::Clubs, Suit::Diamonds, Suit::Hearts] {
            // Number values
            for i in 2..=10 {
                deck.push(Card {
                    rank: Rank::Number(i),
                    suit,
                })
            }

            // Face card values
            for &rank in &[Rank::Jack, Rank::Queen, Rank::King, Rank::Ace] {
                deck.push(Card { rank, suit });
            }
        }

        shuffle(&mut deck);
        Self { cards: deck }
    }

    fn draw(&mut self) -> Option<Card> {
        self.cards.pop()
    }
}

pub struct Hand {
    cards: Vec<Card>,
}

impl Hand {
    pub fn new() -> Self {
        Self { cards: Vec::new() }
    }

    pub fn value(&self) -> u32 {
        let mut aces = 0;
        let mut total_value = 0;

        for &card in &self.cards {
            match card.rank {
                Rank::Number(value) => {
                    total_value += value;
                }
                Rank::Ace => {
                    aces += 1;
                    total_value += 11;
                }
                // Jack, Queen, King
                _ => {
                    total_value += 10;
                }
            }
        }

        while total_value > 21 && aces > 0 {
            total_value -= 10; // Treat the ace as a 1 instead
            aces -= 1;
        }

        total_value
    }

    pub fn add_card(&mut self, card: Card) {
        self.cards.push(card);
    }

    pub fn cards(&self) -> &[Card] {
        &self.cards
    }
}

#[derive(Debug, Clone, Copy)]
pub enum GameOutcome {
    PlayerBusted,
    DealerBusted,
    PlayerWins,
    DealerWins,
    Tie,
}

#[derive(Debug, Clone, Copy)]
pub enum GameState {
    WaitingPlayerChoice,
    DealerTurn,
    Over(GameOutcome),
}

pub struct Blackjack {
    deck: Deck,
    player_hand: Hand,
    dealer_hand: Hand,
    state: GameState,
}

impl Blackjack {
    pub fn new() -> Self {
        let mut deck = Deck::new_shuffled();
        let mut player_hand = Hand::new();
        let mut dealer_hand = Hand::new();

        // Deal initial cards
        player_hand.add_card(deck.draw().unwrap());
        player_hand.add_card(deck.draw().unwrap());
        dealer_hand.add_card(deck.draw().unwrap());
        dealer_hand.add_card(deck.draw().unwrap());

        Self {
            deck,
            player_hand,
            dealer_hand,
            state: GameState::WaitingPlayerChoice,
        }
    }

    fn set_state(&mut self, state: GameState) {
        self.state = state;
    }

    pub fn state(&self) -> &GameState {
        &self.state
    }

    pub fn dealer_cards(&self) -> &[Card] {
        self.dealer_hand.cards()
    }

    pub fn player_cards(&self) -> &[Card] {
        self.player_hand.cards()
    }

    pub fn player_hand_value(&self) -> u32 {
        self.player_hand.value()
    }

    pub fn dealer_hand_value(&self) -> u32 {
        self.dealer_hand.value()
    }

    fn deal_player_card(&mut self) -> Option<Card> {
        let card = self.deck.draw()?;
        self.player_hand.add_card(card);
        Some(card.clone())
    }

    pub fn deal_dealer_cards(&mut self) {
        while self.dealer_hand.value() < 17 {
            if let Some(card) = self.deck.draw() {
                self.dealer_hand.add_card(card);
            }
        }

        let dealer_hand_value = self.dealer_hand_value();
        let player_hand_value = self.player_hand_value();

        if dealer_hand_value > 21 {
            self.set_state(GameState::Over(GameOutcome::DealerBusted));
        } else if player_hand_value > dealer_hand_value {
            self.set_state(GameState::Over(GameOutcome::PlayerWins));
        } else if player_hand_value < dealer_hand_value {
            self.set_state(GameState::Over(GameOutcome::DealerWins));
        } else {
            self.set_state(GameState::Over(GameOutcome::Tie));
        }
    }

    pub fn hit(&mut self) -> Option<Card> {
        let card = self.deal_player_card()?;

        if self.player_hand_value() > 21 {
            self.set_state(GameState::Over(GameOutcome::PlayerBusted));
        }

        Some(card)
    }

    pub fn stand(&mut self) {
        self.set_state(GameState::DealerTurn);
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BetResult {
    Win,       // regular win, pays 1:1
    Blackjack, // natural two-card 21, pays 3:2
    Push,      // tie, bet returned
    Lose,      // loss or bust, bet forfeited
}

/*
Settle a round. `bankroll` is the money on hand AFTER the wager has already
been moved into the pot, `bet` is that wager. Returns the new bankroll and the
net change from the player's perspective (positive = profit, negative = loss).

Blackjack pays 3:2 rounded down to the whole dollar (house-favour rounding, as
on lower-limit tables). Any leftover smaller than the $5 minimum bet gets swept
by the out-of-chips restart.
*/
pub fn settle_bet(bankroll: u32, bet: u32, result: BetResult) -> (u32, i64) {
    match result {
        BetResult::Win => (bankroll + bet * 2, bet as i64),
        BetResult::Blackjack => {
            let bonus = bet * 3 / 2; // 3:2, integer division rounds down
            (bankroll + bet + bonus, bonus as i64)
        }
        BetResult::Push => (bankroll + bet, 0),
        BetResult::Lose => (bankroll, -(bet as i64)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn win_returns_stake_plus_equal_winnings() {
        // Started $100, bet $40 (hand now $60), then won 1:1.
        assert_eq!(settle_bet(60, 40, BetResult::Win), (140, 40));
    }

    #[test]
    fn blackjack_pays_three_to_two_rounded_down() {
        // $40 natural -> +$60 profit (3:2 exactly).
        assert_eq!(settle_bet(60, 40, BetResult::Blackjack), (160, 60));
        // $5 natural -> $7.50 rounded down to $7.
        assert_eq!(settle_bet(95, 5, BetResult::Blackjack), (107, 7));
        // $25 natural -> $37.50 rounded down to $37.
        assert_eq!(settle_bet(75, 25, BetResult::Blackjack), (137, 37));
    }

    #[test]
    fn push_returns_the_bet_unchanged() {
        assert_eq!(settle_bet(60, 40, BetResult::Push), (100, 0));
    }

    #[test]
    fn loss_keeps_bankroll_flat_and_reports_the_loss() {
        assert_eq!(settle_bet(60, 40, BetResult::Lose), (60, -40));
    }

    #[test]
    fn net_change_matches_the_bankroll_delta() {
        // The reported net always equals the actual change in on-hand money,
        // remembering the wager was removed before settling.
        for &bet in &[5u32, 25, 40, 100] {
            for result in [
                BetResult::Win,
                BetResult::Blackjack,
                BetResult::Push,
                BetResult::Lose,
            ] {
                let before = 500;
                let (after, net) = settle_bet(before, bet, result);
                // before the round the player had `before + bet` on hand
                assert_eq!(
                    after as i64 - (before as i64 + bet as i64),
                    net,
                    "net {} disagrees with delta (bet {}, {:?})",
                    net,
                    bet,
                    result
                );
            }
        }
    }
}
