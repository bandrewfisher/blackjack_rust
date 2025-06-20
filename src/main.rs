mod blackjack;
mod blackjack_gui;

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
    let gui = BlackjackGui::new();
    gui.run().await;
}
