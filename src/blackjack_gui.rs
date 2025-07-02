use crate::blackjack::Blackjack;
use crate::sheet_sprite::{SheetSprite, SharedSprite};
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
}

impl BlackjackGui {
    fn deck_pos() -> Vec2 {
        vec2(
            screen_width() - SCREEN_PADDING - (DECK_W * SPRITE_SCALE),
            SCREEN_PADDING
        )
    }
    pub async fn new() -> Self {
        let mut sprites: Vec<SharedSprite> = Vec::new();

        // Load deck texture and sprite
        let deck_texture = Rc::new(load_texture("assets/cards/decks_fixed.png").await.unwrap());
        deck_texture.set_filter(FilterMode::Nearest);

        let deck_pos = BlackjackGui::deck_pos();
        let mut deck_sprite = SheetSprite::new(
            deck_texture,
            4,
            0,
            DECK_W,
            DECK_H,
            deck_pos
        );
        deck_sprite.set_scale(SPRITE_SCALE);
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
                let mut card_sprite = SheetSprite::new(
                    Rc::clone(&cards_texture),
                    col,
                    row,
                    CARD_W,
                    CARD_H,
                    vec2(0.0, 0.0)
                );
                card_sprite.set_scale(2.0);
                card_sprites.insert(format!("{}{}", rank, suit), Rc::new(RefCell::new(card_sprite)));
            }
        }

        Self {
            game: Blackjack::new(),
            card_sprites,
            deal_animations: AnimationQueue::new(),
            sprites,
            state: GameState::NewGame
        }
    }

    fn handle_state(&mut self) {
        match self.state {
            GameState::NewGame => {
                let player_cards = self.game.player_cards();
                let dealer_cards = self.game.dealer_cards();

                let deck_pos = BlackjackGui::deck_pos();
                let pcard1 = self.card_sprites.get(&player_cards[0].repr()).unwrap();
                let pcard2 = self.card_sprites.get(&player_cards[1].repr()).unwrap();

                // pcard1.borrow_mut().set_pos(BlackjackGui::deck_pos());
                // pcard2.borrow_mut().set_pos(BlackjackGui::deck_pos());

                self.deal_animations.push(
                    Animation::new(
                        Rc::clone(pcard1),
                        vec2(200.0, 200.0),
                        1.0
                    )
                );
                self.deal_animations.push(
                    Animation::new(
                        Rc::clone(pcard2),
                        vec2(220.0, 200.0),
                        1.0
                    )
                );
                self.sprites.push(Rc::clone(pcard1));
                self.sprites.push(Rc::clone(pcard2));
                self.state = GameState::DealingInitialHand;
            }
            
            _ => {}
        }
    }

    pub async fn run(&mut self) {
        loop {
            let delta_time = get_frame_time();

            // Handle current game state
            self.handle_state();

            // Rest of the code handles drawing to the screen
            clear_background(RED);

            // Render deal animations
            self.deal_animations.tick(delta_time);

            // Render sprites
            for sprite in &self.sprites {
                sprite.borrow().draw();
            }

            next_frame().await
        }
    }
}
