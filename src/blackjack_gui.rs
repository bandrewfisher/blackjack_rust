use crate::blackjack::Blackjack;
use crate::sheet_sprite::SheetSprite;
use std::rc::Rc;

use macroquad::prelude::*;
use std::collections::HashMap;

const SCREEN_PADDING: f32 = 25.0;
/*
How will we render deal animations?
Well, we can have a vector of animations.
Each animation will have:
- state: not_started, running, complete
- start xy, destination xy, cur xy
- speed
- sprite
- texture
- on_complete callback

After animation is complete (desination == cur),
then remove it from the vector
*/
enum AnimationState {
    NotStarted,
    Running,
    Complete,
}

struct Animation {
    state: AnimationState,
    start_pos: Vec2,
    end_pos: Vec2,
    cur_pos: Vec2,
    duration_secs: f32, // yes it sounds like sex
    sprite: Rc<SheetSprite>
}

impl Animation {
    fn new(sprite: Rc<SheetSprite>, start_pos: Vec2, end_pos: Vec2, duration_secs: f32) -> Self {
        Self {
            sprite,
            state: AnimationState::NotStarted,
            start_pos,
            end_pos,
            cur_pos: start_pos.clone(),
            duration_secs,
        }
    }

    fn tick(&mut self, delta_time: f32) {
        match self.state {
            AnimationState::NotStarted => {
                // Draw at the initial position
                self.sprite.draw(
                    self.start_pos.x, self.start_pos.y
                );
                // Now transfer to running state
                self.state = AnimationState::Running;
                
            }
            AnimationState::Running => {
                let delta_x = self.end_pos.x - self.start_pos.x;
                let delta_y = self.end_pos.y - self.start_pos.y;

                self.cur_pos.x += delta_x * delta_time / self.duration_secs;
                self.cur_pos.y += delta_y * delta_time / self.duration_secs;

                // Clamp at the end in case we've gone past
                let traveled_x = self.cur_pos.x - self.start_pos.x;
                let traveled_y = self.cur_pos.y - self.start_pos.y;
                
                if (traveled_x + traveled_y) > (delta_x + delta_y) { // Manhattan distance
                    self.cur_pos.x = self.end_pos.x;
                    self.cur_pos.y = self.end_pos.y;

                    self.state = AnimationState::Complete;
                }

                self.sprite.draw(
                    self.cur_pos.x, self.cur_pos.y
                );
            }
            AnimationState::Complete => {
                self.sprite.draw(
                    self.cur_pos.x, self.cur_pos.y
                );
            }
        }
    }
}

pub struct BlackjackGui {
    game: Blackjack,
    card_sprites: HashMap<String, Rc<SheetSprite>>,
    deck_sprite: SheetSprite,
    deal_animations: Vec<Animation>,
}

impl BlackjackGui {
    pub async fn new() -> Self {
        // Load deck texture and sprite
        let deck_width = 49.0;
        let deck_height = 73.0;
        let deck_texture = Rc::new(load_texture("assets/cards/decks_fixed.png").await.unwrap());
        deck_texture.set_filter(FilterMode::Nearest);

        let mut deck_sprite = SheetSprite::new(deck_texture, 4, 0, deck_width, deck_height);
        deck_sprite.set_scale(2.0);

        // Load card texture and sprites
        let cards_texture = Rc::new(load_texture("assets/cards/cards.png").await.unwrap());
        cards_texture.set_filter(FilterMode::Nearest);

        let card_width = 48.0;
        let card_height = 64.0;
        let mut card_sprites = HashMap::new();
        for (row, suit) in ["H", "D", "S", "C"].iter().enumerate() {
            for (col, rank) in [
                "A", "2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K",
            ]
            .iter()
            .enumerate()
            {
                let mut card_sprite =
                    SheetSprite::new(Rc::clone(&cards_texture), col, row, card_width, card_height);
                card_sprite.set_scale(2.0);
                card_sprites.insert(format!("{}{}", rank, suit), Rc::new(card_sprite));
            }
        }

        Self {
            game: Blackjack::new(),
            card_sprites,
            deck_sprite,
            deal_animations: Vec::new(),
        }
    }

    fn get_deck_pos(&self) -> Vec2 {
        vec2(
            screen_width() - SCREEN_PADDING - self.deck_sprite.width(),
            SCREEN_PADDING,
        )
    }

    /// Show the deck in the top right corner of the screen
    pub fn render_deck(&self) {
        let deck_pos = self.get_deck_pos();
        self.deck_sprite
            .draw(deck_pos.x, deck_pos.y);
    }

    pub async fn run(&mut self) {
        if let Some(card_sprite) = self.card_sprites.get("KS") {
            self.deal_animations.push(
                Animation::new(
                    Rc::clone(card_sprite),
                    self.get_deck_pos(),
                    vec2(
                        (screen_width() / 2.) - (card_sprite.width() / 2.),
                        screen_height() - card_sprite.height() -  SCREEN_PADDING,
                    ),
                    0.75
                )
            );
        }

        println!("start {}", self.deal_animations[0].start_pos);
        println!("end {}", self.deal_animations[0].end_pos);

        
        loop {
            let delta_time = get_frame_time();

            clear_background(RED);

            // Render deck
            self.render_deck();

            // Render deal animations
            for animation in &mut self.deal_animations {
                animation.tick(delta_time);
            }
            // if let Some(animation) = self.deal_animations.get(0) {
            //     animation.tick(delta_time);
            // }
            next_frame().await
        }
    }
}
