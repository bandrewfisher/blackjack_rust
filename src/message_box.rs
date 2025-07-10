use macroquad::prelude::*;
use crate::button::{Button, ButtonEvent, ButtonConfig, measure_button};

const MODAL_HEIGHT: f32 = 250.0;
const MODAL_WIDTH: f32 = 500.0;
const MODAL_PADDING: f32 = 32.0;

const TITLE_TEXT_SIZE: f32 = 48.0;
const MESSAGE_TEXT_SIZE: f32 = 32.0;

pub struct MessageBoxEvent {
    pub button_event: ButtonEvent
}

pub struct MessageBox {
    pub title: String,
    pub messages: Vec<String>,
    pub button: Button,
    rect: Rect
}

impl MessageBox {
    pub fn new(title: &str, messages: Vec<String>, button_text: &str) -> Self {
        let rect = Rect::new(
            screen_width() / 2.0 - MODAL_WIDTH / 2.0,
            screen_height() / 2.0 - MODAL_HEIGHT / 2.0,
            MODAL_WIDTH,
            MODAL_HEIGHT,
        );
        
        let button_config = ButtonConfig::default();
        let button_dims = measure_button(button_text, &button_config);
        Self {
            title: String::from(title),
            messages,
            rect,
            button: Button::new(
                button_text,
                rect.x + rect.w / 2.0 - button_dims.0 / 2.0,
                rect.y + MODAL_HEIGHT - button_dims.1 - MODAL_PADDING,
                button_config
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

        let title_size = measure_text(
            &self.title,
            None,
            TITLE_TEXT_SIZE as u16,
            1.0
        );

        // Draw title
        let title_pos = vec2(
           self.rect.x + self.rect.w / 2.0 - title_size.width / 2.0,
           self.rect.y + MODAL_PADDING + title_size.height,
        );
        draw_text(
            &self.title,
            title_pos.x,
            title_pos.y,
            TITLE_TEXT_SIZE,
            WHITE
        );

        // Draw messages
        let message_spacing = 16.0;
        for (idx, message) in self.messages.iter().enumerate() {
            let message_size = measure_text(
                message,
                None,
                MESSAGE_TEXT_SIZE as u16,
                1.0
            );
            draw_text(
                message,
                self.rect.x + self.rect.w / 2.0 - message_size.width / 2.0,
                title_pos.y + message_spacing * (idx + 1) as f32 + message_size.height * (idx + 1) as f32,
                MESSAGE_TEXT_SIZE,
                WHITE
            );
        }

        // Draw button
        let button_event = self.button.draw();

        MessageBoxEvent {
            button_event 
        }
    }
}
