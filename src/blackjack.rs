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

mod utils {
    use super::{Card, Rank, Suit};
    use rand::prelude::SliceRandom;

    pub fn create_deck() -> Vec<Card> {
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

        deck
    }

    pub fn hand_value(hand: &Vec<Card>) -> u32 {
        let mut aces = 0;
        let mut total_value = 0;

        for &card in hand {
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

    pub fn shuffle_deck(deck: &mut Vec<Card>) {
        let mut rng = rand::rng();
        deck.shuffle(&mut rng);
    }
}

#[derive(Debug, Clone, Copy)]
pub enum GameState {
    WaitingPlayerChoice,
    PlayerBusted,
    DealerTurn,
    DealerBusted,
    PlayerWins,
    DealerWins,
    Tie
}

pub struct Blackjack {
    deck: Vec<Card>,
    player_cards: Vec<Card>,
    dealer_cards: Vec<Card>,
    state: GameState,
}

impl Blackjack {
    pub fn new() -> Self {
        // Create the shuffled deck
        let mut deck = utils::create_deck();
        utils::shuffle_deck(&mut deck);

        // Deal the player cards
        let player_cards = vec![deck.pop().unwrap(), deck.pop().unwrap()];

        // Deal the dealer cards
        let dealer_cards = vec![deck.pop().unwrap(), deck.pop().unwrap()];

        Self {
            deck,
            player_cards,
            dealer_cards,
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
        &self.dealer_cards
    }

    pub fn player_cards(&self) -> &[Card] {
        &self.player_cards
    }

    pub fn player_hand_value(&self) -> u32 {
        utils::hand_value(&self.player_cards)
    }

    pub fn dealer_hand_value(&self) -> u32 {
        utils::hand_value(&self.dealer_cards)
    }

    fn deal_player_card(&mut self) -> Card {
        let card = self.deck.pop().unwrap();
        self.player_cards.push(card);
        
        card.clone()
    }

    pub fn deal_dealer_cards(&mut self) {
        while self.dealer_hand_value() < 17 {
            let card = self.deck.pop().unwrap();
            self.dealer_cards.push(card);
        } 

        let dealer_hand_value = self.dealer_hand_value();
        let player_hand_value = self.player_hand_value();
        
        if dealer_hand_value > 21 {
            self.set_state(GameState::DealerBusted);
        } else if player_hand_value > dealer_hand_value {
            self.set_state(GameState::PlayerWins);
        } else if player_hand_value < dealer_hand_value {
            self.set_state(GameState::DealerWins);
        } else {
            self.set_state(GameState::Tie);
        }
    }

    pub fn hit(&mut self) -> Card {
        let card = self.deal_player_card();

        if self.player_hand_value() > 21 {
            self.set_state(GameState::PlayerBusted);
        }

        card
    }

    pub fn stand(&mut self) {
        self.set_state(GameState::DealerTurn);
    }
}
