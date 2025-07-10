use macroquad::prelude::*;

const BUTTON_FONT_SIZE: u16 = 32;
const BUTTON_PADDING_X: f32 = 12.0;
const BUTTON_PADDING_Y: f32 = 8.0;

const BUTTON_BORDER_PX: f32 = 4.0;

pub struct ButtonEvent {
    pub clicked: bool,
    pub hovered: bool,
}

pub struct ButtonConfig {
    pub color: Color,
    pub text_color: Color,
    pub border_color: Color,
}

impl Default for ButtonConfig {
    fn default() -> Self {
        Self {
            color: LIGHTGRAY,
            text_color: BLACK,
            border_color: BLACK,
        }
    }
}

pub fn measure_button(label: &str, config: &ButtonConfig) -> (f32, f32) {
    //! Returns a tuple of (width, height) in pixels of a button
    let text_size = measure_text(label, None, BUTTON_FONT_SIZE, 1.0);
    let width = text_size.width + (BUTTON_PADDING_X * 2.0);
    let height = text_size.height + (BUTTON_PADDING_Y * 2.0);

    (width, height)
}

pub struct Button {
    label: String,
    rect: Rect,
    text_size: TextDimensions,
    config: ButtonConfig,
}

impl Button {
    pub fn new(label: &str, x: f32, y: f32, config: ButtonConfig) -> Self {
        let text_size = measure_text(label, None, BUTTON_FONT_SIZE, 1.0);
        let button_dims = measure_button(label, &config);

        let rect = Rect::new(
            x,
            y,
            button_dims.0,
            button_dims.1,
        );
        Self {
            label: String::from(label),
            rect,
            text_size,
            config,
        }
    }

    pub fn height(&self) -> f32 {
        self.rect.h
    }

    pub fn width(&self) -> f32 {
        self.rect.w
    }

    pub fn draw(&self) -> ButtonEvent {
        let mouse_pos = mouse_position();
        let hovered = self.rect.contains(vec2(mouse_pos.0, mouse_pos.1));

        // Draw border
        draw_rectangle(
            self.rect.x - BUTTON_BORDER_PX,
            self.rect.y - BUTTON_BORDER_PX,
            self.rect.w + (BUTTON_BORDER_PX * 2.0),
            self.rect.h + (BUTTON_BORDER_PX * 2.0),
            self.config.border_color,
        );

        // Draw background
        draw_rectangle(
            self.rect.x,
            self.rect.y,
            self.rect.w,
            self.rect.h,
            self.config.color,
        );

        // Draw text
        let text_x = self.rect.x + BUTTON_PADDING_X;
        let text_y = self.rect.y + BUTTON_PADDING_Y + self.text_size.height;
        draw_text(&self.label, text_x, text_y, BUTTON_FONT_SIZE as f32, self.config.text_color);

        ButtonEvent {
            hovered,
            clicked: hovered && is_mouse_button_pressed(MouseButton::Left),
        }
    }
}
