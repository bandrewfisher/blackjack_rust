use crate::animation::{Animation, AnimationQueue, AnimationState};
use crate::blackjack::{Blackjack, Card, Hand};
use crate::button::{Button, ButtonConfig, ButtonEvent};
use crate::message_box::MessageBox;
use crate::sheet_sprite::{SharedSprite, Sprite};
use std::cell::RefCell;
use std::rc::Rc;

use macroquad::audio::{PlaySoundParams, Sound, load_sound, play_sound};
use macroquad::prelude::*;
use std::collections::{HashMap, VecDeque};

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

#[derive(Debug, Clone, PartialEq)]
enum GameState {
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

    // Queue for deal animations
    deal_animations: AnimationQueue<CardAnimationMetadata>,

    // Each sprite to render on the screen
    deck_sprite: SharedSprite,
    card_sprites: Vec<SharedSprite>,
    dealer_down_card: Option<SharedSprite>,

    // Textures
    deck_texture: Rc<Texture2D>,
    cards_texture: Rc<Texture2D>,

    // Sounds
    card_flip_sound: Sound,

    // Buttons
    hit_button: Button,
    stand_button: Button,

    // Hands
    player_hand: Hand,
    dealer_hand: Hand,

    // Message box, set to None if there is not one to display
    message_box: Option<MessageBox>,
}

impl BlackjackGui {
    pub async fn new() -> Self {
        let mut card_sprites: Vec<SharedSprite> = Vec::new();

        // Load deck texture and sprite
        let deck_texture = Rc::new(load_texture("assets/cards/decks_fixed.png").await.unwrap());
        deck_texture.set_filter(FilterMode::Nearest);

        let deck_pos = deck_pos();
        let mut deck_sprite = Rc::new(RefCell::new(Sprite::new(
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

        // Load SFX
        let card_flip_sound = load_sound("assets/sfx/flipcard.wav").await.unwrap();

        // Create buttons
        let hit_btn_pos = vec2(SCREEN_PADDING + 50.0, deck_pos.y + 50.0);
        let hit_button = Button::new(
            "Hit",
            hit_btn_pos.x,
            hit_btn_pos.y,
            ButtonConfig {
                color: GREEN,
                ..Default::default()
            },
        );

        let stand_button = Button::new(
            "Stand",
            hit_btn_pos.x,
            hit_btn_pos.y + hit_button.height() + 25.0,
            ButtonConfig {
                color: RED,
                text_color: WHITE,
                ..Default::default()
            },
        );

        Self {
            game: Blackjack::new(),
            card_sprite_map,
            deal_animations: AnimationQueue::new(),
            card_sprites,
            deck_sprite,
            state: GameState::NewGame,
            deck_texture,
            cards_texture,
            card_flip_sound,
            hit_button,
            stand_button,
            player_hand: Hand::new(),
            dealer_hand: Hand::new(),
            message_box: None,
            dealer_down_card: None,
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
            let mut new_sprite = Sprite::new(
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

    fn get_score_messages(&self) -> Vec<String> {
        //! Returns the message to show in each modal, which shows the scores
        let mut score_messages = Vec::new();
        score_messages.push(format!("Your cards: {}", self.player_hand.value()));
        score_messages.push(format!("Dealer cards: {}", self.dealer_hand.value()));

        score_messages
    }

    fn handle_state(&mut self) {
        match self.state {
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
                    self.set_state(GameState::WaitingPlayerInput);
                }
            }

            GameState::WaitingPlayerInput => {
                if self.player_hand.value() > 21 {
                    self.message_box = Some(MessageBox::new(
                        "You busted!",
                        self.get_score_messages(),
                        "Play again",
                    ));
                    self.set_state(GameState::PlayerBusted);
                }
            }

            GameState::PlayerBusted => {}

            GameState::DealingDealerCards => {
                if self.deal_animations.len() < 1 {
                    let dealer_hand_value = self.dealer_hand.value();
                    let player_hand_value = self.player_hand.value();

                    let score_messages = self.get_score_messages();

                    if dealer_hand_value > 21 {
                        self.message_box = Some(MessageBox::new(
                            "Dealer busted, you win!",
                            score_messages,
                            "Play again",
                        ));
                        self.set_state(GameState::DealerBusted);
                    } else if dealer_hand_value > player_hand_value {
                        self.message_box =
                            Some(MessageBox::new("You lose!", score_messages, "Play again"));
                        self.set_state(GameState::DealerWins);
                    } else if player_hand_value > dealer_hand_value {
                        self.message_box =
                            Some(MessageBox::new("You win!", score_messages, "Play again"));
                        self.set_state(GameState::PlayerWins);
                    } else if player_hand_value == dealer_hand_value {
                        self.message_box =
                            Some(MessageBox::new("It's a tie!", score_messages, "Play again"));
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

    fn handle_buttons(&mut self) {
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

        // For some reason on Mac, the click isn't always registered???
        // Works fine on Linux + WASM. Moral of the story - FUCK APPLE OMFGGGGG
        // Stand button
        let stand_event = self.stand_button.draw();
        if stand_event.clicked && self.state == GameState::WaitingPlayerInput {
            // Remove the facedown dealer card
            let down_card_sprite = Rc::clone(self.dealer_down_card.as_ref().unwrap());
            let down_card_pos = down_card_sprite.borrow().get_pos();
            self.remove_card_sprite(&down_card_sprite);

            // Show the second dealer card
            let dealer_down_card = self.game.dealer_cards()[1].repr();
            if let Some(card_sprite) = self.create_card_sprite(&dealer_down_card, down_card_pos) {
                self.card_sprites.push(card_sprite);
                self.dealer_hand
                    .add_card(Card::from_repr(&dealer_down_card));
            }

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

    pub fn reset(&mut self) {
        self.game = Blackjack::new();
        self.set_state(GameState::NewGame);
        self.deal_animations = AnimationQueue::new();
        self.player_hand = Hand::new();
        self.dealer_hand = Hand::new();
        self.message_box = None;
        self.card_sprites = Vec::new();
    }

    pub fn handle_message_box(&mut self) {
        if let Some(message_box) = &self.message_box {
            let message_box_event = message_box.draw();
            if message_box_event.button_event.clicked {
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

            // Draw buttons
            self.handle_buttons();

            // Render score
            self.draw_player_score();
            self.draw_dealer_score();

            // Render sprites
            self.deck_sprite.borrow().draw();
            for sprite in &self.card_sprites {
                sprite.borrow().draw();
            }

            // Render message box
            self.handle_message_box();

            next_frame().await
        }
    }
}
