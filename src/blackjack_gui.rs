use crate::animation::{Animation, AnimationQueue, AnimationState};
use crate::blackjack::Blackjack;
use crate::sheet_sprite::{SharedSprite, Sprite};
use std::cell::RefCell;
use std::rc::Rc;

use macroquad::audio::{PlaySoundParams, Sound, load_sound, play_sound};
use macroquad::prelude::*;
use std::collections::{HashMap, VecDeque};

const SCREEN_PADDING: f32 = 25.0;
const SPRITE_SCALE: f32 = 2.0;

const DECK_W: f32 = 49.0;
const DECK_H: f32 = 73.0;

const CARD_W: f32 = 48.0;
const CARD_H: f32 = 64.0;

fn deck_pos() -> Vec2 {
    vec2(
        screen_width() - SCREEN_PADDING - (DECK_W * SPRITE_SCALE),
        SCREEN_PADDING,
    )
}

fn player_cards_pos() -> Vec2 {
    vec2(
        (screen_width() / 2.0) - (CARD_W * SPRITE_SCALE / 2.0),
        screen_height() - SCREEN_PADDING - (CARD_H * SPRITE_SCALE),
    )
}

fn dealer_cards_pos() -> Vec2 {
    vec2(
        (screen_width() / 2.0) - (CARD_W * SPRITE_SCALE / 2.0),
        SCREEN_PADDING,
    )
}

enum GameState {
    NewGame,
    DealingInitialHand,
    WaitingPlayerInput,
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
    card_sprites: HashMap<String, SharedSprite>,
    deal_animations: AnimationQueue<CardAnimationMetadata>,
    sprites: Vec<SharedSprite>,

    // Textures
    deck_texture: Rc<Texture2D>,
    cards_texture: Rc<Texture2D>,

    // Sounds
    card_flip_sound: Sound,
}

impl BlackjackGui {
    pub async fn new() -> Self {
        let mut sprites: Vec<SharedSprite> = Vec::new();

        // Load deck texture and sprite
        let deck_texture = Rc::new(load_texture("assets/cards/decks_fixed.png").await.unwrap());
        deck_texture.set_filter(FilterMode::Nearest);

        let deck_pos = deck_pos();
        let mut deck_sprite = Sprite::new(
            Rc::clone(&deck_texture),
            4,
            0,
            DECK_W,
            DECK_H,
            deck_pos,
            SPRITE_SCALE,
        );
        sprites.push(Rc::new(RefCell::new(deck_sprite)));

        // Load card texture and sprites
        let cards_texture = Rc::new(load_texture("assets/cards/cards.png").await.unwrap());
        cards_texture.set_filter(FilterMode::Nearest);

        let mut card_sprites = HashMap::new();
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
                    CARD_W,
                    CARD_H,
                    vec2(0.0, 0.0),
                    SPRITE_SCALE,
                );
                card_sprites.insert(
                    format!("{}{}", rank, suit),
                    Rc::new(RefCell::new(card_sprite)),
                );
            }
        }

        // Load SFX
        let card_flip_sound = load_sound("assets/sfx/flipcard.wav").await.unwrap();

        Self {
            game: Blackjack::new(),
            card_sprites,
            deal_animations: AnimationQueue::new(),
            sprites,
            state: GameState::NewGame,
            deck_texture,
            cards_texture,
            card_flip_sound,
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
            CARD_W,
            CARD_H,
            position,
            SPRITE_SCALE,
        )))
    }

    fn create_card_sprite(&self, card_repr: &str, position: Vec2) -> Option<SharedSprite> {
        if let Some(card_sprite_ref) = self.card_sprites.get(card_repr) {
            let card_sprite = card_sprite_ref.borrow();
            let mut new_sprite = Sprite::new(
                Rc::clone(&self.cards_texture),
                card_sprite.col(),
                card_sprite.row(),
                CARD_W,
                CARD_H,
                position,
                SPRITE_SCALE,
            );

            return Some(Rc::new(RefCell::new(new_sprite)));
        }
        None
    }

    fn remove_sprite(&mut self, sprite: &SharedSprite) {
        self.sprites.retain(|s| !Rc::ptr_eq(s, sprite))
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

                // Player card animations
                self.deal_animations.push(Animation::new(
                    Rc::clone(&pcard1),
                    player_cards_pos,
                    0.7,
                    CardAnimationMetadata {
                        is_dealer_card: false,
                        card_repr: pcard1_repr,
                        should_flip: true,
                    },
                ));
                self.deal_animations.push(Animation::new(
                    Rc::clone(&pcard2),
                    vec2(
                        player_cards_pos.x + (CARD_W * SPRITE_SCALE / 3.0),
                        player_cards_pos.y,
                    ),
                    0.7,
                    CardAnimationMetadata {
                        is_dealer_card: false,
                        card_repr: pcard2_repr,
                        should_flip: true,
                    },
                ));

                // Dealer card animations
                self.deal_animations.push(Animation::new(
                    Rc::clone(&dcard1),
                    dealer_cards_pos,
                    0.7,
                    CardAnimationMetadata {
                        is_dealer_card: true,
                        card_repr: dcard1_repr,
                        should_flip: true,
                    },
                ));
                self.deal_animations.push(Animation::new(
                    Rc::clone(&dcard2),
                    vec2(
                        dealer_cards_pos.x + (CARD_W * SPRITE_SCALE / 3.0),
                        dealer_cards_pos.y,
                    ),
                    0.7,
                    CardAnimationMetadata {
                        is_dealer_card: true,
                        card_repr: dcard2_repr,
                        should_flip: false,
                    },
                ));

                self.sprites.push(pcard1);
                self.sprites.push(pcard2);
                self.sprites.push(dcard1);
                self.sprites.push(dcard2);

                self.set_state(GameState::DealingInitialHand);
            }

            GameState::DealingInitialHand => {
                // Done dealing the cards
                if self.deal_animations.len() < 1 {
                    self.set_state(GameState::WaitingPlayerInput);
                }
            }

            GameState::WaitingPlayerInput => {}
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
            println!("starting animation");
            play_sound(
                &self.card_flip_sound,
                PlaySoundParams {
                    looped: false,
                    volume: 0.7,
                },
            );
        }

        // We get an animation back if ticking yields a completed animation
        if let Some(animation) = self.deal_animations.tick(delta_time) {
            let sprite = animation.sprite();
            let metadata = animation.metadata();

            if !metadata.should_flip {
                return;
            }

            self.remove_sprite(sprite);

            if let Some(card_sprite) =
                self.create_card_sprite(&metadata.card_repr, sprite.borrow().get_pos())
            {
                self.sprites.push(card_sprite);
            }
        }
    }

    pub async fn run(&mut self) {
        loop {
            let delta_time = get_frame_time();

            // Handle current game state
            self.handle_state();

            // Rest of the code handles drawing to the screen
            clear_background(RED);

            // Handle deal animations
            self.handle_deal_animations(delta_time);

            // Render sprites
            for sprite in &self.sprites {
                sprite.borrow().draw();
            }

            next_frame().await
        }
    }
}
