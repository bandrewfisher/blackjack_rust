use crate::blackjack::Blackjack;
use crate::sheet_sprite::{Sprite, SharedSprite};
use crate::animation::{Animation, AnimationQueue};
use std::rc::Rc;
use std::cell::RefCell;

use macroquad::prelude::*;
use std::collections::{HashMap};

const SCREEN_PADDING: f32 = 25.0;
const SPRITE_SCALE: f32 = 2.0;

const DECK_W: f32 = 49.0;
const DECK_H: f32 = 73.0;

const CARD_W: f32 = 48.0;
const CARD_H: f32 = 64.0;

fn deck_pos() -> Vec2 {
    vec2(
        screen_width() - SCREEN_PADDING - (DECK_W * SPRITE_SCALE),
        SCREEN_PADDING
    )
}

fn player_cards_pos() -> Vec2 {
    vec2(
        (screen_width() / 2.0) - (CARD_W * SPRITE_SCALE / 2.0),
        screen_height() - SCREEN_PADDING - (CARD_H * SPRITE_SCALE)
    )
}

enum GameState {
    NewGame,
    DealingInitialHand
}

pub struct BlackjackGui {
    game: Blackjack,
    state: GameState,
    card_sprites: HashMap<String, SharedSprite>,
    deal_animations: AnimationQueue,
    sprites: Vec<SharedSprite>,

    // Textures
    deck_texture: Rc<Texture2D>,
    cards_texture: Rc<Texture2D>
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
            SPRITE_SCALE
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
                    SPRITE_SCALE
                );
                card_sprites.insert(format!("{}{}", rank, suit), Rc::new(RefCell::new(card_sprite)));
            }
        }

        Self {
            game: Blackjack::new(),
            card_sprites,
            deal_animations: AnimationQueue::new(),
            sprites,
            state: GameState::NewGame,
            deck_texture,
            cards_texture
        }
    }

    fn set_state(&mut self, state: GameState) {
        self.state = state;
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
                SPRITE_SCALE
            );

            return Some(Rc::new(RefCell::new(new_sprite)));
        }
        None
    }

    fn handle_state(&mut self) {
        match self.state {
            GameState::NewGame => {
                let player_cards = self.game.player_cards();
                let dealer_cards = self.game.dealer_cards();

                let deck_pos = deck_pos();
                let player_cards_pos = player_cards_pos();

                let pcard1 = self.create_card_sprite(&player_cards[0].repr(), deck_pos).unwrap();
                let pcard2 = self.create_card_sprite(&player_cards[1].repr(), deck_pos).unwrap();

                self.deal_animations.push(
                    Animation::new(
                        Rc::clone(&pcard1),
                        player_cards_pos,
                        1.0
                    )
                );
                self.deal_animations.push(
                    Animation::new(
                        Rc::clone(&pcard2),
                        vec2(player_cards_pos.x + (CARD_W * SPRITE_SCALE / 3.0), player_cards_pos.y),
                        1.0
                    )
                );
                self.sprites.push(pcard1);
                self.sprites.push(pcard2);

                self.set_state(GameState::DealingInitialHand);
            }

            GameState::DealingInitialHand => {}
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
            self.deal_animations.tick(delta_time);

            // Render sprites
            for sprite in &self.sprites {
                sprite.borrow().draw();
            }

            next_frame().await
        }
    }
}
