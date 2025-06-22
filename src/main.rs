mod blackjack;
mod blackjack_gui;
mod sheet_sprite;

use macroquad::prelude::*;
use blackjack_gui::BlackjackGui;


fn window_conf() -> Conf {
    Conf {
        window_title: "Blackjack".to_owned(),
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut gui = BlackjackGui::new().await;
    gui.run().await;
}
