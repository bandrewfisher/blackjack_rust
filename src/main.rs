use rand::seq::SliceRandom;

use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute, queue, terminal,
};
use std::io::{self, Write};

#[derive(Debug, Clone, Copy)]
enum Suit {
    Spades,
    Clubs,
    Diamonds,
    Hearts,
}

#[derive(Debug, Clone, Copy)]
enum Rank {
    Number(u32),
    Jack,
    Queen,
    King,
    Ace,
}

#[derive(Debug, Clone, Copy)]
struct Card {
    suit: Suit,
    rank: Rank,
}

impl Card {
    fn repr(&self) -> String {
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

fn create_deck() -> Vec<Card> {
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

fn hand_value(hand: &Vec<Card>) -> u32 {
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
        total_value -= 10;  // Treat the ace as a 1 instead
        aces -= 1;
    }

    total_value
}

fn shuffle_deck(deck: &mut Vec<Card>) {
    let mut rng = rand::rng();
    deck.shuffle(&mut rng);
}

struct Blackjack {
    deck: Vec<Card>,
    player_cards: Vec<Card>,
    dealer_cards: Vec<Card>,
}

impl Blackjack {
    fn new() -> Self {
        // Create the shuffled deck
        let mut deck = create_deck();
        shuffle_deck(&mut deck);

        // Deal the player cards
        let player_cards = vec![deck.pop().unwrap(), deck.pop().unwrap()];

        // Deal the dealer cards
        let dealer_cards = vec![deck.pop().unwrap(), deck.pop().unwrap()];

        Self {
            deck,
            player_cards,
            dealer_cards,
        }
    }
}

struct BlackjackCli {
    blackjack: Blackjack
}

impl BlackjackCli {
    fn new() -> Self {
        Self {
            blackjack: Blackjack::new()
        }
    }

    fn show_cards(&self) {
        println!("Dealer card: {}", &self.blackjack.dealer_cards[0].repr());

        print!("Your cards: ");
        for &card in &self.blackjack.player_cards {
            print!("{} ", card.repr());
        }
        println!("\nHand value: {}", hand_value(&self.blackjack.player_cards));
    }

    fn run(&mut self) -> io::Result<()> {
        println!("BLACKJACK");

        // Show the player's cards
        self.show_cards();

        // Ask to hit or stay
        print!("Do you want to (h)it or (s)tay? ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let choice = input.trim().to_lowercase();
        match choice.as_str() {
            "h" => {
                println!("hit");
                self.blackjack.player_cards.push(
                    self.blackjack.deck.pop().unwrap()
                );
                self.show_cards();
            }
            "s" => {
                println!("stay");
            }
            _ => {}
        }

        Ok(())
    }
}

fn main() -> io::Result<()> {
    let mut blackjack = BlackjackCli::new();
    blackjack.run();
    Ok(())
    // let mut stdout = io::stdout();
    // execute!(stdout, terminal::EnterAlternateScreen)?;
    // terminal::enable_raw_mode()?;
    // execute!(stdout, cursor::Hide)?;
    //
    // loop {
    //     // Clear the screen
    //     queue!(stdout, terminal::Clear(terminal::ClearType::All),)?;
    //
    //     // Show available space
    //     queue!(stdout, cursor::MoveTo(0, 0))?;
    //     let (cols, rows) = terminal::size()?;
    //     print!("cols {} rows {}", cols, rows);
    //     queue!(stdout, cursor::MoveToNextLine(1))?;
    //
    //     // Print a grid
    //     for row in 0..30 {
    //         for col in 0..100 {
    //             if col == 0 {
    //                 print!("{row}");
    //             } else {
    //                 print!(".");
    //             }
    //         }
    //         queue!(stdout, cursor::MoveToNextLine(1))?;
    //     }
    //
    //     stdout.flush()?;
    //
    //     // if event::poll(Duration::from_millis(500))? {
    //     if let Event::Key(key_event) = event::read()? {
    //         match key_event.code {
    //             KeyCode::Char('q') => {
    //                 break;
    //             }
    //             // KeyCode::Right => {
    //             //     print!("Going right")
    //             // }
    //             _ => {}
    //         }
    //     }
    //     // }
    // }
    //
    // execute!(stdout, cursor::Show)?;
    // terminal::disable_raw_mode()?;
    // execute!(stdout, terminal::LeaveAlternateScreen)?;
    // Ok(())
}
