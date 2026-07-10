use crate::animation::{Animation, AnimationQueue, AnimationState};
use crate::blackjack::{BetResult, Blackjack, Card, Hand, settle_bet};
use crate::button::{Button, ButtonConfig};
use crate::message_box::MessageBox;
use crate::sheet_sprite::{SharedSprite, Sprite};
use std::cell::RefCell;
use std::rc::Rc;

use macroquad::audio::{PlaySoundParams, Sound, load_sound, play_sound};
use macroquad::prelude::*;
use std::collections::HashMap;

const SCREEN_PADDING: f32 = 25.0;
const SPRITE_SCALE: f32 = 2.0;

const DECK_W_PX: f32 = 49.0;
const DECK_H_PX: f32 = 73.0;

const DECK_W: f32 = DECK_W_PX * SPRITE_SCALE;
const DECK_H: f32 = DECK_H_PX * SPRITE_SCALE;

const CARD_W_PX: f32 = 48.0;
const CARD_H_PX: f32 = 64.0;

const CARD_W: f32 = CARD_W_PX * SPRITE_SCALE;
const CARD_H: f32 = CARD_H_PX * SPRITE_SCALE;

const CARD_GAP: f32 = CARD_W / 4.0; // How much space to put between each card

const SCORE_FONT_SIZE: f32 = 32.0;
const READOUT_FONT_SIZE: f32 = 32.0;

// Chips (source sprite is 46x48 in the sheet)
const CHIP_WIDTH_PX: f32 = 46.0;
const CHIP_HEIGHT_PX: f32 = 48.0;

// Chip heap rendering
const HEAP_CHIP_SCALE: f32 = 1.0;
const HEAP_CHIP_W: f32 = CHIP_WIDTH_PX * HEAP_CHIP_SCALE;
const HEAP_CHIP_H: f32 = CHIP_HEIGHT_PX * HEAP_CHIP_SCALE;
const CHIP_UNIT: u32 = 5; // one drawn chip is worth $5
const HEAP_MAX_STACK: usize = 5; // chips tall per stack before starting a new one
const HEAP_MAX_STACKS: usize = 6; // stacks wide before the pile visually maxes out

// Money
const STARTING_BANKROLL: u32 = 100;

// Floating money-change text
const FLOAT_TTL: f32 = 1.4;
const FLOAT_RISE: f32 = 55.0;
const FLOAT_FONT_SIZE: f32 = 44.0;

fn deck_pos() -> Vec2 {
    vec2(screen_width() - SCREEN_PADDING - DECK_W, SCREEN_PADDING)
}

fn player_cards_pos() -> Vec2 {
    vec2(
        (screen_width() / 2.0) - (CARD_W / 2.0),
        screen_height() - SCREEN_PADDING - CARD_H,
    )
}

fn dealer_cards_pos() -> Vec2 {
    vec2((screen_width() / 2.0) - (CARD_W / 2.0), SCREEN_PADDING)
}

fn hand_heap_base() -> Vec2 {
    // Bottom-left: the chips in your hand
    vec2(SCREEN_PADDING + 110.0, screen_height() - SCREEN_PADDING - 10.0)
}

fn hand_readout_pos() -> Vec2 {
    vec2(SCREEN_PADDING, screen_height() - SCREEN_PADDING - 130.0)
}

fn pot_heap_base() -> Vec2 {
    // Center of the table: the pot
    vec2(screen_width() / 2.0, screen_height() / 2.0 + 40.0)
}

// A short-lived "+$40" / "-$25" that rises off the bankroll and fades.
struct FloatingText {
    text: String,
    pos: Vec2,
    color: Color,
    elapsed: f32,
}

impl FloatingText {
    fn update_and_draw(&mut self, dt: f32) -> bool {
        self.elapsed += dt;
        self.pos.y -= FLOAT_RISE * dt;

        let alpha = (1.0 - self.elapsed / FLOAT_TTL).clamp(0.0, 1.0);
        let mut color = self.color;
        color.a = alpha;

        let size = measure_text(&self.text, None, FLOAT_FONT_SIZE as u16, 1.0);
        draw_text(
            &self.text,
            self.pos.x - size.width / 2.0,
            self.pos.y,
            FLOAT_FONT_SIZE,
            color,
        );

        self.elapsed < FLOAT_TTL
    }
}

#[derive(Debug, Clone, PartialEq)]
enum GameState {
    Betting,
    NewGame,
    DealingCard,
    WaitingPlayerInput,
    DealingDealerCards,
    PlayerBusted,
    DealerBusted,
    DealerWins,
    PlayerWins,
    Tie,
}

#[derive(Debug, Clone)]
struct CardAnimationMetadata {
    card_repr: String,
    should_flip: bool,
    is_dealer_card: bool,
}

pub struct BlackjackGui {
    game: Blackjack,
    state: GameState,

    // Maps a card repr like "AH" to a sprite template for that card
    card_sprite_map: HashMap<String, SharedSprite>,

    // Money
    bankroll: u32,
    bet: u32,
    last_net: i64,
    out_of_chips: bool,
    money_float: Option<FloatingText>,

    // A single red chip, repositioned + redrawn to build heaps
    red_chip: SharedSprite,

    // Queue for deal animations
    deal_animations: AnimationQueue<CardAnimationMetadata>,

    // Each sprite to render on the screen
    deck_sprite: SharedSprite,
    card_sprites: Vec<SharedSprite>,
    dealer_down_card: Option<SharedSprite>,

    // Textures
    deck_texture: Rc<Texture2D>,
    cards_texture: Rc<Texture2D>,
    chips_texture: Rc<Texture2D>,

    // Sounds
    card_flip_sound: Sound,

    // Buttons - play phase
    hit_button: Button,
    stand_button: Button,

    // Buttons - betting phase
    plus5_button: Button,
    plus25_button: Button,
    clear_bet_button: Button,
    deal_button: Button,

    // Hands
    player_hand: Hand,
    dealer_hand: Hand,

    // Message box, set to None if there is not one to display
    message_box: Option<MessageBox>,
}

impl BlackjackGui {
    pub async fn new() -> Self {
        let card_sprites: Vec<SharedSprite> = Vec::new();

        // Load deck texture and sprite
        let deck_texture = Rc::new(load_texture("assets/cards/decks_fixed.png").await.unwrap());
        deck_texture.set_filter(FilterMode::Nearest);

        let deck_pos = deck_pos();
        let deck_sprite = Rc::new(RefCell::new(Sprite::new(
            Rc::clone(&deck_texture),
            4,
            0,
            DECK_W_PX,
            DECK_H_PX,
            deck_pos,
            SPRITE_SCALE,
        )));

        // Load card texture and sprites
        let cards_texture = Rc::new(load_texture("assets/cards/cards.png").await.unwrap());
        cards_texture.set_filter(FilterMode::Nearest);

        let mut card_sprite_map = HashMap::new();
        for (row, suit) in ["H", "D", "S", "C"].iter().enumerate() {
            for (col, rank) in [
                "A", "2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K",
            ]
            .iter()
            .enumerate()
            {
                let card_sprite = Sprite::new(
                    Rc::clone(&cards_texture),
                    col,
                    row,
                    CARD_W_PX,
                    CARD_H_PX,
                    vec2(0.0, 0.0),
                    SPRITE_SCALE,
                );
                card_sprite_map.insert(
                    format!("{}{}", rank, suit),
                    Rc::new(RefCell::new(card_sprite)),
                );
            }
        }

        // Load chips texture and the single red chip used to build heaps
        let chips_texture = Rc::new(load_texture("assets/cards/chips.png").await.unwrap());
        chips_texture.set_filter(FilterMode::Nearest);
        let red_chip = Rc::new(RefCell::new(Sprite::new(
            Rc::clone(&chips_texture),
            0,
            0,
            CHIP_WIDTH_PX,
            CHIP_HEIGHT_PX,
            vec2(0.0, 0.0),
            HEAP_CHIP_SCALE,
        )));

        // Load SFX
        let card_flip_sound = load_sound("assets/sfx/flipcard.wav").await.unwrap();

        // Play-phase buttons (top-left)
        let btn_x = SCREEN_PADDING + 50.0;
        let btn_y = deck_pos.y + 50.0;
        let hit_button = Button::new(
            "Hit",
            btn_x,
            btn_y,
            ButtonConfig {
                color: GREEN,
                ..Default::default()
            },
        );
        let stand_button = Button::new(
            "Stand",
            btn_x,
            btn_y + hit_button.height() + 25.0,
            ButtonConfig {
                color: RED,
                text_color: WHITE,
                ..Default::default()
            },
        );

        // Betting-phase buttons (same corner, shown only while betting)
        let plus5_button = Button::new(
            "+$5",
            btn_x,
            btn_y,
            ButtonConfig {
                color: LIGHTGRAY,
                ..Default::default()
            },
        );
        let bh = plus5_button.height();
        let gap = 20.0;
        let plus25_button = Button::new(
            "+$25",
            btn_x,
            btn_y + (bh + gap),
            ButtonConfig {
                color: LIGHTGRAY,
                ..Default::default()
            },
        );
        let clear_bet_button = Button::new(
            "Clear Bet",
            btn_x,
            btn_y + 2.0 * (bh + gap),
            ButtonConfig {
                color: RED,
                text_color: WHITE,
                ..Default::default()
            },
        );
        let deal_button = Button::new(
            "Deal",
            btn_x,
            btn_y + 3.0 * (bh + gap),
            ButtonConfig {
                color: GREEN,
                ..Default::default()
            },
        );

        Self {
            game: Blackjack::new(),
            card_sprite_map,
            deal_animations: AnimationQueue::new(),
            card_sprites,
            deck_sprite,
            state: GameState::Betting,
            deck_texture,
            cards_texture,
            chips_texture,
            card_flip_sound,
            hit_button,
            stand_button,
            plus5_button,
            plus25_button,
            clear_bet_button,
            deal_button,
            player_hand: Hand::new(),
            dealer_hand: Hand::new(),
            message_box: None,
            dealer_down_card: None,

            bankroll: STARTING_BANKROLL,
            bet: 0,
            last_net: 0,
            out_of_chips: false,
            money_float: None,
            red_chip,
        }
    }

    fn set_state(&mut self, state: GameState) {
        self.state = state;
    }

    fn create_facedown_card_sprite(&self, position: Vec2) -> SharedSprite {
        Rc::new(RefCell::new(Sprite::new(
            Rc::clone(&self.cards_texture),
            4,
            4,
            CARD_W_PX,
            CARD_H_PX,
            position,
            SPRITE_SCALE,
        )))
    }

    fn create_card_sprite(&self, card_repr: &str, position: Vec2) -> Option<SharedSprite> {
        if let Some(card_sprite_ref) = self.card_sprite_map.get(card_repr) {
            let card_sprite = card_sprite_ref.borrow();
            let new_sprite = Sprite::new(
                Rc::clone(&self.cards_texture),
                card_sprite.col(),
                card_sprite.row(),
                CARD_W_PX,
                CARD_H_PX,
                position,
                SPRITE_SCALE,
            );

            return Some(Rc::new(RefCell::new(new_sprite)));
        }
        None
    }

    fn remove_card_sprite(&mut self, sprite: &SharedSprite) {
        self.card_sprites.retain(|s| !Rc::ptr_eq(s, sprite))
    }

    // Flip the dealer's face-down card up and fold it into the dealer hand.
    fn reveal_dealer_down_card(&mut self) {
        if let Some(down_card) = self.dealer_down_card.take() {
            let pos = down_card.borrow().get_pos();
            self.remove_card_sprite(&down_card);

            let repr = self.game.dealer_cards()[1].repr();
            if let Some(card_sprite) = self.create_card_sprite(&repr, pos) {
                self.card_sprites.push(card_sprite);
                self.dealer_hand.add_card(Card::from_repr(&repr));
            }
        }
    }

    fn get_score_messages(&self) -> Vec<String> {
        //! Returns the message to show in each modal, which shows the scores
        let mut score_messages = Vec::new();
        score_messages.push(format!("Your cards: {}", self.player_hand.value()));
        score_messages.push(format!("Dealer cards: {}", self.dealer_hand.value()));

        score_messages
    }

    fn payout_message(&self) -> String {
        if self.last_net > 0 {
            format!("You won ${}!", self.last_net)
        } else if self.last_net < 0 {
            format!("You lost ${}", -self.last_net)
        } else {
            String::from("Push - bet returned")
        }
    }

    // Move the wager into the bankroll per the outcome and spawn the money float.
    fn resolve_bet(&mut self, result: BetResult) {
        let (new_bankroll, net) = settle_bet(self.bankroll, self.bet, result);
        self.bankroll = new_bankroll;
        self.last_net = net;

        if net != 0 {
            let base = hand_readout_pos();
            let (text, color) = if net > 0 {
                (format!("+${}", net), GREEN)
            } else {
                (format!("-${}", -net), RED)
            };
            self.money_float = Some(FloatingText {
                text,
                pos: vec2(base.x + 90.0, base.y - 20.0),
                color,
                elapsed: 0.0,
            });
        }

        self.bet = 0;
    }

    // Build a modal from a title plus the score lines and the payout line.
    fn show_outcome(&mut self, title: &str) {
        let mut messages = self.get_score_messages();
        messages.push(self.payout_message());
        self.message_box = Some(MessageBox::new(title, messages, "Play again"));
    }

    fn handle_state(&mut self) {
        match self.state {
            GameState::Betting => {
                // Nothing left to bet with and no wager down - offer a restart.
                if self.bankroll < CHIP_UNIT && self.bet == 0 && self.message_box.is_none() {
                    self.message_box = Some(MessageBox::new(
                        "You're out of chips!",
                        vec![String::from("Time to start over.")],
                        "Start over",
                    ));
                    self.out_of_chips = true;
                }
            }

            GameState::NewGame => {
                let player_cards = self.game.player_cards();
                let dealer_cards = self.game.dealer_cards();

                let deck_pos = deck_pos();
                let player_cards_pos = player_cards_pos();
                let dealer_cards_pos = dealer_cards_pos();

                // Player card sprites and card strings
                let pcard1_repr = player_cards[0].repr();
                let pcard2_repr = player_cards[1].repr();
                let pcard1 = self.create_facedown_card_sprite(deck_pos);
                let pcard2 = self.create_facedown_card_sprite(deck_pos);

                // Dealer card sprites and card strings
                let dcard1_repr = dealer_cards[0].repr();
                let dcard2_repr = dealer_cards[1].repr();
                let dcard1 = self.create_facedown_card_sprite(deck_pos);
                let dcard2 = self.create_facedown_card_sprite(deck_pos);

                self.dealer_down_card = Some(Rc::clone(&dcard2));
                // Player card animations
                self.deal_animations.push(Animation::new(
                    pcard1,
                    player_cards_pos,
                    0.7,
                    CardAnimationMetadata {
                        is_dealer_card: false,
                        card_repr: pcard1_repr,
                        should_flip: true,
                    },
                ));
                self.deal_animations.push(Animation::new(
                    pcard2,
                    vec2(player_cards_pos.x + CARD_GAP, player_cards_pos.y),
                    0.7,
                    CardAnimationMetadata {
                        is_dealer_card: false,
                        card_repr: pcard2_repr,
                        should_flip: true,
                    },
                ));

                // Dealer card animations
                self.deal_animations.push(Animation::new(
                    dcard1,
                    dealer_cards_pos,
                    0.7,
                    CardAnimationMetadata {
                        is_dealer_card: true,
                        card_repr: dcard1_repr,
                        should_flip: true,
                    },
                ));
                self.deal_animations.push(Animation::new(
                    dcard2,
                    vec2(dealer_cards_pos.x + CARD_GAP, dealer_cards_pos.y),
                    0.7,
                    CardAnimationMetadata {
                        is_dealer_card: true,
                        card_repr: dcard2_repr,
                        should_flip: false,
                    },
                ));

                self.set_state(GameState::DealingCard);
            }

            GameState::DealingCard => {
                // Done dealing the cards
                if self.deal_animations.len() < 1 {
                    // A natural (two-card 21) settles immediately - no hit/stand.
                    if self.player_hand.cards().len() == 2 && self.player_hand.value() == 21 {
                        self.reveal_dealer_down_card();
                        if self.dealer_hand.value() == 21 {
                            // Dealer also has a natural - push.
                            self.resolve_bet(BetResult::Push);
                            self.show_outcome("Push!");
                            self.set_state(GameState::Tie);
                        } else {
                            self.resolve_bet(BetResult::Blackjack);
                            self.show_outcome("Blackjack!");
                            self.set_state(GameState::PlayerWins);
                        }
                    } else {
                        self.set_state(GameState::WaitingPlayerInput);
                    }
                }
            }

            GameState::WaitingPlayerInput => {
                if self.player_hand.value() > 21 {
                    self.resolve_bet(BetResult::Lose);
                    self.show_outcome("You busted!");
                    self.set_state(GameState::PlayerBusted);
                }
            }

            GameState::PlayerBusted => {}

            GameState::DealingDealerCards => {
                if self.deal_animations.len() < 1 {
                    let dealer_hand_value = self.dealer_hand.value();
                    let player_hand_value = self.player_hand.value();
                    let is_natural =
                        self.player_hand.cards().len() == 2 && player_hand_value == 21;
                    let win_result = if is_natural {
                        BetResult::Blackjack
                    } else {
                        BetResult::Win
                    };

                    if dealer_hand_value > 21 {
                        self.resolve_bet(win_result);
                        self.show_outcome("You win!");
                        self.set_state(GameState::DealerBusted);
                    } else if dealer_hand_value > player_hand_value {
                        self.resolve_bet(BetResult::Lose);
                        self.show_outcome("You lose!");
                        self.set_state(GameState::DealerWins);
                    } else if player_hand_value > dealer_hand_value {
                        self.resolve_bet(win_result);
                        self.show_outcome("You win!");
                        self.set_state(GameState::PlayerWins);
                    } else {
                        self.resolve_bet(BetResult::Push);
                        self.show_outcome("It's a tie!");
                        self.set_state(GameState::Tie);
                    }
                }
            }

            GameState::DealerBusted => {}

            GameState::DealerWins => {}

            GameState::PlayerWins => {}

            GameState::Tie => {}
        }
    }

    /*
    Handle each deal animation. We'll deal the card at the front of the
    queue. After it's reached its final destination, we'll flip it over
    and start handling the next deal animation.
    */
    pub fn handle_deal_animations(&mut self, delta_time: f32) {
        // If animation is starting, play the card flip sound effect
        if self
            .deal_animations
            .cur_animation_state()
            .is_some_and(|s| s == AnimationState::NotStarted)
        {
            play_sound(
                &self.card_flip_sound,
                PlaySoundParams {
                    looped: false,
                    volume: 0.7,
                },
            );

            // Add animation sprite to the sprites queue.
            // We do it here to keep the correct rendering order, so that
            // each dealt card appears on top of the prior ones.
            if let Some(animation) = self.deal_animations.cur_animation() {
                self.card_sprites.push(Rc::clone(animation.sprite()));
            }
        }

        // We get an animation back if ticking yields a completed animation
        if let Some(animation) = self.deal_animations.tick(delta_time) {
            let sprite = animation.sprite();
            let metadata = animation.metadata();

            // Don't flip over dealer second card
            if !metadata.should_flip {
                return;
            }

            // Remove the facedown card and show the faceup card instead
            self.remove_card_sprite(sprite);
            if let Some(card_sprite) =
                self.create_card_sprite(&metadata.card_repr, sprite.borrow().get_pos())
            {
                self.card_sprites.push(card_sprite);
            }

            // Update the player hand with the new card.
            // We don't use it from Blackjack because we only want
            // to show the updated score when the card is flipped over
            if metadata.is_dealer_card {
                self.dealer_hand
                    .add_card(Card::from_repr(&metadata.card_repr));
            } else {
                self.player_hand
                    .add_card(Card::from_repr(&metadata.card_repr));
            }
        }
    }

    fn add_to_bet(&mut self, amount: u32) {
        if self.bankroll >= amount {
            self.bankroll -= amount;
            self.bet += amount;
        }
    }

    fn handle_betting_buttons(&mut self) {
        if self.plus5_button.draw().clicked {
            self.add_to_bet(5);
        }
        if self.plus25_button.draw().clicked {
            self.add_to_bet(25);
        }
        if self.clear_bet_button.draw().clicked {
            self.bankroll += self.bet;
            self.bet = 0;
        }
        // Can't deal a $0 hand
        if self.deal_button.draw().clicked && self.bet > 0 {
            self.set_state(GameState::NewGame);
        }
    }

    fn handle_play_buttons(&mut self) {
        // Hit button
        let hit_event = self.hit_button.draw();
        if hit_event.clicked && self.state == GameState::WaitingPlayerInput {
            if let Some(card) = self.game.hit() {
                let player_cards_pos = player_cards_pos();
                let new_card_pos = vec2(
                    player_cards_pos.x + (self.game.player_cards().len() - 1) as f32 * CARD_GAP, // Slide the card over by however many other cards are already there
                    player_cards_pos.y,
                );
                let card_sprite = self.create_facedown_card_sprite(deck_pos());
                // Add a new animation for the card
                self.deal_animations.push(Animation::new(
                    card_sprite,
                    new_card_pos,
                    0.7,
                    CardAnimationMetadata {
                        is_dealer_card: false,
                        card_repr: card.repr(),
                        should_flip: true,
                    },
                ));

                self.set_state(GameState::DealingCard);
            }
        }

        // Stand button
        let stand_event = self.stand_button.draw();
        if stand_event.clicked && self.state == GameState::WaitingPlayerInput {
            // Flip the dealer's hole card up
            self.reveal_dealer_down_card();

            // Add each new dealer card to the animation queue
            self.game.deal_dealer_cards();
            let dealer_cards_pos = dealer_cards_pos();
            let mut card_gap_idx = 1; // 2 cards already in dealer's hand, -1 for 0-based index
            for card in &self.game.dealer_cards()[2..] {
                let card_sprite = self.create_card_sprite(&card.repr(), deck_pos()).unwrap();
                card_gap_idx += 1;
                self.deal_animations.push(Animation::new(
                    card_sprite,
                    vec2(
                        dealer_cards_pos.x + CARD_GAP * card_gap_idx as f32,
                        dealer_cards_pos.y,
                    ),
                    0.7,
                    CardAnimationMetadata {
                        is_dealer_card: true,
                        card_repr: card.repr(),
                        should_flip: true,
                    },
                ));
            }

            self.set_state(GameState::DealingDealerCards);
        }
    }

    pub fn draw_player_score(&self) {
        let player_cards_pos = player_cards_pos();
        draw_text(
            &format!("Score: {}", self.player_hand.value()),
            player_cards_pos.x - 32.0,
            player_cards_pos.y - 16.0,
            SCORE_FONT_SIZE,
            BLACK,
        );
    }

    pub fn draw_dealer_score(&self) {
        let dealer_cards_pos = dealer_cards_pos();
        let text = format!("Score: {}", self.dealer_hand.value());
        let text_size = measure_text(&text, None, SCORE_FONT_SIZE as u16, 1.0);
        draw_text(
            &text,
            dealer_cards_pos.x - 32.0,
            dealer_cards_pos.y + text_size.height + CARD_H + 16.0,
            SCORE_FONT_SIZE,
            BLACK,
        );
    }

    // Draw a clustered heap of red chips worth `amount`, anchored at the
    // bottom-center `base`. Stacks overlap and the middle is bumped up a chip
    // so it reads as a real pile, not a bar chart.
    fn draw_chip_heap(&self, amount: u32, base: Vec2) {
        let total = ((amount / CHIP_UNIT) as usize).min(HEAP_MAX_STACK * HEAP_MAX_STACKS);
        if total == 0 {
            return;
        }

        let num_stacks = ((total + HEAP_MAX_STACK - 1) / HEAP_MAX_STACK).max(1);
        let mut heights = vec![total / num_stacks; num_stacks];
        let remainder = total % num_stacks;
        // Give the leftover chips to the central stacks -> taller in the middle.
        let start = (num_stacks - remainder) / 2;
        for k in 0..remainder {
            heights[start + k] += 1;
        }

        let stack_dx = HEAP_CHIP_W * 0.5;
        let chip_dy = HEAP_CHIP_H * 0.22;
        let total_w = stack_dx * (num_stacks as f32 - 1.0);
        let start_x = base.x - (total_w + HEAP_CHIP_W) / 2.0;

        for (i, &h) in heights.iter().enumerate() {
            let sx = start_x + i as f32 * stack_dx;
            for c in 0..h {
                let y = base.y - HEAP_CHIP_H - c as f32 * chip_dy;
                self.red_chip.borrow_mut().set_pos(vec2(sx, y));
                self.red_chip.borrow().draw();
            }
        }
    }

    fn draw_money_readouts(&self) {
        let hand_pos = hand_readout_pos();
        draw_text(
            &format!("On hand: ${}", self.bankroll),
            hand_pos.x,
            hand_pos.y,
            READOUT_FONT_SIZE,
            BLACK,
        );

        if self.state == GameState::Betting {
            let hint = "Place your bet";
            let size = measure_text(hint, None, READOUT_FONT_SIZE as u16, 1.0);
            draw_text(
                hint,
                screen_width() / 2.0 - size.width / 2.0,
                screen_height() / 2.0 - 130.0,
                READOUT_FONT_SIZE,
                BLACK,
            );
        }

        if self.bet > 0 || self.state == GameState::Betting {
            let text = format!("Bet: ${}", self.bet);
            let size = measure_text(&text, None, READOUT_FONT_SIZE as u16, 1.0);
            draw_text(
                &text,
                screen_width() / 2.0 - size.width / 2.0,
                screen_height() / 2.0 - 80.0,
                READOUT_FONT_SIZE,
                BLACK,
            );
        }
    }

    pub fn reset(&mut self) {
        self.game = Blackjack::new();
        self.set_state(GameState::Betting);
        self.deal_animations = AnimationQueue::new();
        self.player_hand = Hand::new();
        self.dealer_hand = Hand::new();
        self.message_box = None;
        self.card_sprites = Vec::new();
        self.dealer_down_card = None;
        self.bet = 0;
    }

    pub fn handle_message_box(&mut self) {
        if let Some(message_box) = &self.message_box {
            let message_box_event = message_box.draw();
            if message_box_event.button_event.clicked {
                if self.out_of_chips {
                    self.bankroll = STARTING_BANKROLL;
                    self.out_of_chips = false;
                }
                self.reset();
            }
        }
    }

    pub async fn run(&mut self) {
        // Main loop
        loop {
            let delta_time = get_frame_time();

            // Handle current game state
            self.handle_state();

            // Rest of the code handles drawing to the screen
            clear_background(VIOLET);

            // Handle deal animations
            self.handle_deal_animations(delta_time);

            // Draw the buttons for the current phase
            if self.state == GameState::Betting {
                self.handle_betting_buttons();
            } else {
                self.handle_play_buttons();
            }

            // Render score (only once cards are on the table)
            if self.state != GameState::Betting {
                self.draw_player_score();
                self.draw_dealer_score();
            }

            // Render card sprites
            self.deck_sprite.borrow().draw();
            for sprite in &self.card_sprites {
                sprite.borrow().draw();
            }

            // Render chip heaps: your hand and the pot
            self.draw_chip_heap(self.bankroll, hand_heap_base());
            self.draw_chip_heap(self.bet, pot_heap_base());

            // Money readouts + betting hint
            self.draw_money_readouts();

            // Floating money-change text
            if let Some(float) = &mut self.money_float {
                if !float.update_and_draw(delta_time) {
                    self.money_float = None;
                }
            }

            // Render message box on top
            self.handle_message_box();

            next_frame().await
        }
    }
}
