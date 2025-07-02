use macroquad::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;

pub struct SheetSprite {
    source_rect: Rect,
    scale: f32,
    texture: Rc<Texture2D>,
    position: Vec2,
}

pub type SharedSprite = Rc<RefCell<SheetSprite>>;

impl SheetSprite {
    pub fn new(
        texture: Rc<Texture2D>,
        col: usize,
        row: usize,
        card_width: f32,
        card_height: f32,
        position: Vec2
    ) -> Self {
        let source_rect = Rect::new(
            col as f32 * card_width,
            row as f32 * card_height,
            card_width,
            card_height,
        );
        Self {
            source_rect,
            scale: 1.0,
            texture,
            position,
        }
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

    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
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
