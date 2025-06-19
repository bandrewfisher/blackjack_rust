mod blackjack;

use blackjack::{Blackjack, GameOutcome, GameState};
use std::io::{self, Write};

struct BlackjackCli {
    blackjack: Blackjack,
}

impl BlackjackCli {
    fn new() -> Self {
        Self {
            blackjack: Blackjack::new(),
        }
    }

    fn show_cards(&self) {
        println!("Dealer card: {}", &self.blackjack.dealer_cards()[0].repr());

        print!("Your cards: ");
        for &card in self.blackjack.player_cards() {
            print!("{} ", card.repr());
        }
        println!("\nHand value: {}", self.blackjack.player_hand_value());
    }

    fn show_dealer_cards(&self, num_cards: usize) {
        print!("Dealer cards: ");

        for &card in &self.blackjack.dealer_cards()[..num_cards] {
            print!("{} ", card.repr());
        }
        println!(
            "\nDealer hand value: {}",
            self.blackjack.dealer_hand_value()
        );
    }

    fn run(&mut self) -> io::Result<()> {
        println!("BLACKJACK");

        loop {
            match *self.blackjack.state() {
                GameState::WaitingPlayerChoice => {
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
                            // let card = self.blackjack.hit();
                            // println!("You got a {}", card.repr());
                            if let Some(card) = self.blackjack.hit() {
                                println!("You got a {}", card.repr());
                            }
                        }
                        "s" => {
                            self.blackjack.stand();
                        }
                        _ => {}
                    }
                }

                GameState::DealerTurn => {
                    let player_hand_value = self.blackjack.player_hand_value();
                    println!(
                        "Your hand value is {}. Let's see what the dealer has.",
                        player_hand_value
                    );

                    println!("The dealer reveals their other card.");
                    self.show_dealer_cards(2);
                    self.blackjack.deal_dealer_cards();

                    // Show each dealer card
                    for i in 3..=self.blackjack.dealer_cards().len() {
                        self.show_dealer_cards(i);
                    }
                }

                GameState::Over(outcome) => {
                    match outcome {
                        GameOutcome::PlayerBusted => {
                            self.show_cards();
                            println!("You busted! Game over :(");
                        }

                        GameOutcome::DealerBusted => {
                            println!("Dealer busted, YOU WIN!!");
                        }

                        GameOutcome::PlayerWins => {
                            println!("YOU WIN!!");
                        }

                        GameOutcome::DealerWins => {
                            println!("The dealer beat you :( Told ya the house always wins");
                        }

                        GameOutcome::Tie => {
                            println!("It's a tie! At least you get your money back.");
                        }
                    }
                    break;
                }
            }
        }

        Ok(())
    }
}

fn main() -> io::Result<()> {
    let mut blackjack = BlackjackCli::new();
    blackjack.run()?;

    Ok(())
}
