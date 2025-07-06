use macroquad::prelude::*;
use crate::button::{Button, ButtonEvent, ButtonConfig};

const MODAL_HEIGHT: f32 = 350.0;
const MODAL_WIDTH: f32 = 500.0;

const MESSAGE_TEXT_SIZE: f32 = 64.0;

pub struct MessageBoxEvent {
    pub button_event: ButtonEvent
}

pub struct MessageBox {
    pub message: String,
    pub button: Button,
    rect: Rect
}

impl MessageBox {
    pub fn new(message: &str, button_text: &str) -> Self {
        let rect = Rect::new(
            screen_width() / 2.0 - MODAL_WIDTH / 2.0,
            screen_height() / 2.0 - MODAL_HEIGHT / 2.0,
            MODAL_WIDTH,
            MODAL_HEIGHT,
        );
        
        Self {
            message: String::from(message),
            rect,
            button: Button::new(
                button_text,
                rect.x + rect.w / 2.0,
                rect.y  + rect.h / 2.0,
                ButtonConfig {
                    ..Default::default()
                }
            )
        }
    }

    pub fn draw(&self) -> MessageBoxEvent {
        // Draw background
        draw_rectangle(
            self.rect.x,
            self.rect.y,
            self.rect.w,
            self.rect.h,
            Color::new(0.0, 0.0, 0.0, 0.7)
        );

        let text_size = measure_text(
            &self.message,
            None,
            MESSAGE_TEXT_SIZE as u16,
            1.0
        );

        // Draw message
        draw_text(
            &self.message,
            self.rect.x + self.rect.w / 2.0 - text_size.width / 2.0,
            self.rect.y + 32.0 + text_size.height,
            MESSAGE_TEXT_SIZE,
            WHITE
        );

        let button_event = self.button.draw();

        MessageBoxEvent {
            button_event 
        }
    }
}
