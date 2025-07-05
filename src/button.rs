use macroquad::prelude::*;

const BUTTON_FONT_SIZE: u16 = 32;
const BUTTON_PADDING_X: f32 = 8.0;
const BUTTON_PADDING_Y: f32 = 4.0;

pub struct ButtonEvent {
    pub clicked: bool,
    pub hovered: bool
}

pub struct Button {
    label: String,
    rect: Rect,
    text_size: TextDimensions
}


impl Button {
    pub fn new(label: &str, x: f32, y: f32) -> Self {
        let text_size = measure_text(label, None, BUTTON_FONT_SIZE, 1.0);

        let rect = Rect::new(
            x,
            y,
            text_size.width + (BUTTON_PADDING_X * 2.0),
            text_size.height + (BUTTON_PADDING_Y * 2.0),
        );
        Self {
            label: String::from(label),
            rect,
            text_size
        }
    }

    pub fn draw(&self) -> ButtonEvent {
        let mouse_pos = mouse_position();
        let hovered = self.rect.contains(vec2(mouse_pos.0, mouse_pos.1));

        // Draw background
        draw_rectangle(
            self.rect.x,
            self.rect.y,
            self.rect.w,
            self.rect.h,
            GREEN
        );

        // Draw text
        let text_x = self.rect.x + BUTTON_PADDING_X;
        let text_y = self.rect.y + BUTTON_PADDING_Y + (self.text_size.height);
        draw_text(&self.label, text_x, text_y, BUTTON_FONT_SIZE as f32, BLACK);

        ButtonEvent {
            hovered,
            clicked: hovered && is_mouse_button_pressed(MouseButton::Left)
        }
    }
}
