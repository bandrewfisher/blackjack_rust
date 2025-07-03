use macroquad::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;

pub struct Sprite {
    source_rect: Rect,
    scale: f32,
    texture: Rc<Texture2D>,
    position: Vec2,
    col: usize,
    row: usize
}

pub type SharedSprite = Rc<RefCell<Sprite>>;

impl Sprite {
    pub fn new(
        texture: Rc<Texture2D>,
        col: usize,
        row: usize,
        width: f32,
        height: f32,
        position: Vec2,
        scale: f32
    ) -> Self {
        let source_rect = Rect::new(
            col as f32 * width,
            row as f32 * height,
            width,
            height,
        );
        Self {
            source_rect,
            scale,
            texture,
            position,
            col,
            row
        }
    }

    pub fn row(&self) -> usize {
        self.row
    }

    pub fn col(&self) -> usize {
        self.col
    }

    pub fn get_pos(&self) -> Vec2 {
        self.position
    }

    pub fn set_pos(&mut self, pos: Vec2) {
        self.position = pos;
    }

    pub fn width(&self) -> f32 {
        self.source_rect.w * self.scale
    }

    pub fn height(&self) -> f32 {
        self.source_rect.h * self.scale
    }

    pub fn draw(&self) {
        let dest_size = vec2(
            self.source_rect.w * self.scale,
            self.source_rect.h * self.scale,
        );

        draw_texture_ex(
            &self.texture,
            self.position.x,
            self.position.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(dest_size),
                source: Some(self.source_rect),
                ..Default::default()
            },
        );
    }
}
