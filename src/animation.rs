use crate::sheet_sprite::SharedSprite;

use macroquad::prelude::*;
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy)]
pub enum AnimationState {
    NotStarted,
    Running,
    Complete,
}

pub struct Animation<T> {
    state: AnimationState,
    sprite: SharedSprite,
    start_pos: Vec2,
    end_pos: Vec2,
    duration_secs: f32, // yes it sounds like sex
    metadata: T
}

impl<T> Animation<T> {
    pub fn new(sprite: SharedSprite, end_pos: Vec2, duration_secs: f32, metadata: T) -> Self {
        let start_pos = sprite.borrow().get_pos();
        Self {
            state: AnimationState::NotStarted,
            sprite,
            start_pos,
            end_pos,
            duration_secs,
            metadata
        }
    }

    pub fn sprite(&self) -> &SharedSprite {
        &self.sprite
    }

    pub fn metadata(&self) -> &T {
        &self.metadata
    }

    pub fn state(&self) -> AnimationState {
        self.state
    }

    pub fn tick(&mut self, delta_time: f32) {
        match self.state {
            AnimationState::NotStarted => {
                // Draw at the initial position
                // self.sprite.borrow().draw();
                // Now transfer to running state
                self.state = AnimationState::Running;
            }
            AnimationState::Running => {
                let delta_x = self.end_pos.x - self.start_pos.x;
                let delta_y = self.end_pos.y - self.start_pos.y;

                let mut cur_pos = self.sprite.borrow().get_pos();
                cur_pos.x += delta_x * delta_time / self.duration_secs;
                cur_pos.y += delta_y * delta_time / self.duration_secs;
                self.sprite.borrow_mut().set_pos(cur_pos);

                // Clamp at the end in case we've gone past
                let traveled_x = (cur_pos.x - self.start_pos.x).abs();
                let traveled_y = (cur_pos.y - self.start_pos.y).abs();

                if (traveled_x + traveled_y) > (delta_x.abs() + delta_y.abs()) {
                    // Manhattan distance
                    self.sprite.borrow_mut().set_pos(self.end_pos);
                    self.state = AnimationState::Complete;
                }
            }
            AnimationState::Complete => {}
        }
    }
}

/*
Holds a set of animations in a queue. After each
one is complete, pops the first one and begins
handling the next
*/
pub struct AnimationQueue<T> {
    queue: VecDeque<Animation<T>>,
}

impl<T> AnimationQueue<T> {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    pub fn push(&mut self, animation: Animation<T>) {
        self.queue.push_back(animation);
    }

    pub fn tick(&mut self, delta_time: f32) -> Option<Animation<T>> {
        if let Some(animation) = self.queue.front_mut() {
            if let AnimationState::Complete = animation.state() {
                return self.queue.pop_front();
            } else {
                animation.tick(delta_time);
            }
        }
        None // Queue is empty
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }
}
