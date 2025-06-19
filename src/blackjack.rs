use rand::prelude::SliceRandom;

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

        let mut rng = rand::rng();
        deck.shuffle(&mut rng);

        Self { cards: deck }
    }

    fn draw(&mut self) -> Option<Card> {
        self.cards.pop()
    }
}

struct Hand {
    cards: Vec<Card>,
}

impl Hand {
    fn new() -> Self {
        Self { cards: Vec::new() }
    }

    fn value(&self) -> u32 {
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

    fn add_card(&mut self, card: Card) {
        self.cards.push(card);
    }

    fn cards(&self) -> &[Card] {
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
