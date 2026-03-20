use macroquad::prelude::*;

mod swarm_game;
mod base_game;
mod adq_game;

#[macroquad::main("Game Engine Demo Collection")]
async fn main() {
    loop {
        clear_background(BLACK);

        let center_x = screen_width() / 2.0;
        let center_y = screen_height() / 2.0;

        let title = "Game Engine Demo Collection";
        let title_dim = measure_text(title, None, 40, 1.0);
        draw_text(title, center_x - title_dim.width / 2.0, center_y - 100.0, 40.0, WHITE);

        let option1 = "Press 1: Swarm Survivor (RPG Maker Style)";
        let opt1_dim = measure_text(option1, None, 30, 1.0);
        draw_text(option1, center_x - opt1_dim.width / 2.0, center_y - 20.0, 30.0, ORANGE);

        let option2 = "Press 2: Underground Base Builder (AZITO Style)";
        let opt2_dim = measure_text(option2, None, 30, 1.0);
        draw_text(option2, center_x - opt2_dim.width / 2.0, center_y + 30.0, 30.0, SKYBLUE);

        let option3 = "Press 3: Antigravity Dragon Quest";
        let opt3_dim = measure_text(option3, None, 30, 1.0);
        draw_text(option3, center_x - opt3_dim.width / 2.0, center_y + 80.0, 30.0, PINK);

        if is_key_pressed(KeyCode::Key1) {
            swarm_game::play_swarm_game().await;
        }

        if is_key_pressed(KeyCode::Key2) {
            base_game::play_base_game().await;
        }

        if is_key_pressed(KeyCode::Key3) {
            adq_game::play_adq_game().await;
        }

        next_frame().await;
    }
}
