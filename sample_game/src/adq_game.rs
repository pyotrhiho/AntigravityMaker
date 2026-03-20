use macroquad::prelude::*;

#[derive(Clone, Copy, PartialEq)]
enum GameState {
    Field,
    Dialog,
    Battle,
}

#[derive(Clone, PartialEq)]
struct PlayerStats {
    hp: i32,
    max_hp: i32,
    attack: i32,
}

#[derive(Clone, PartialEq)]
struct EnemyStats {
    name: String,
    hp: i32,
    max_hp: i32,
    attack: i32,
}

pub async fn play_adq_game() {
    let mut state = GameState::Dialog;
    let mut dialog_text = vec![
        "King: Ah, brave hero! The Antigravity Dragon has stolen our gravity!",
        "King: You must travel to the floating castle and defeat it.",
        "King: Be warned, battles require precise timing!",
        "King: Press SPACE when the bar is in the center to strike hard.",
    ];
    let mut dialog_index = 0;

    let mut player_pos = vec2(screen_width() / 2.0, screen_height() / 2.0);

    let mut player_stats = PlayerStats { hp: 50, max_hp: 50, attack: 10 };
    let mut current_enemy = EnemyStats { name: "Slime".to_string(), hp: 20, max_hp: 20, attack: 5 };

    let mut battle_timer = 0.0;
    let mut is_player_turn = true;
    let mut attack_phase = false; // When true, timing bar is active
    let mut timing_bar_pos = 0.0; // 0.0 to 1.0
    let mut timing_bar_dir = 1.0;
    let mut battle_message = "A wild slime appears!".to_string();

    loop {
        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        let dt = get_frame_time();

        clear_background(color_u8!(30, 100, 30, 255)); // Grass green

        match state {
            GameState::Dialog => {
                draw_rectangle(50.0, screen_height() - 150.0, screen_width() - 100.0, 130.0, BLACK);
                draw_rectangle_lines(50.0, screen_height() - 150.0, screen_width() - 100.0, 130.0, 4.0, WHITE);

                if dialog_index < dialog_text.len() {
                    draw_text(dialog_text[dialog_index], 70.0, screen_height() - 80.0, 30.0, WHITE);
                    if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Enter) {
                        dialog_index += 1;
                    }
                } else {
                    state = GameState::Field;
                }
            }
            GameState::Field => {
                // Draw Hero
                draw_circle(player_pos.x, player_pos.y, 20.0, BLUE);

                // Draw King (NPC)
                draw_circle(screen_width() / 2.0, 100.0, 20.0, YELLOW);
                draw_text("King", screen_width() / 2.0 - 20.0, 70.0, 20.0, WHITE);

                let mut move_dir = vec2(0.0, 0.0);
                if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) { move_dir.y -= 1.0; }
                if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) { move_dir.y += 1.0; }
                if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) { move_dir.x -= 1.0; }
                if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) { move_dir.x += 1.0; }

                if move_dir.length_squared() > 0.0 {
                    move_dir = move_dir.normalize();
                    player_pos += move_dir * 150.0 * dt;

                    // Random encounter step
                    if rand::gen_range(0, 1000) < 5 {
                        state = GameState::Battle;
                        if rand::gen_range(0, 2) == 0 {
                            current_enemy = EnemyStats { name: "Floating Slime".to_string(), hp: 20, max_hp: 20, attack: 5 };
                        } else {
                            current_enemy = EnemyStats { name: "Antigravity Bat".to_string(), hp: 30, max_hp: 30, attack: 8 };
                        }
                        battle_message = format!("A wild {} appears!", current_enemy.name);
                        is_player_turn = true;
                        attack_phase = false;
                    }
                }

                // Talk to king again
                if (player_pos - vec2(screen_width() / 2.0, 100.0)).length() < 50.0 && is_key_pressed(KeyCode::Space) {
                    state = GameState::Dialog;
                    dialog_index = 0;
                    dialog_text = vec!["King: Defeat the Antigravity Dragon!"];
                }
            }
            GameState::Battle => {
                clear_background(BLACK);

                // Draw UI
                draw_rectangle(50.0, screen_height() - 150.0, screen_width() - 100.0, 130.0, DARKGRAY);
                draw_rectangle_lines(50.0, screen_height() - 150.0, screen_width() - 100.0, 130.0, 4.0, WHITE);

                // Draw Stats
                draw_text(&format!("HP: {}/{}", player_stats.hp, player_stats.max_hp), 70.0, screen_height() - 100.0, 30.0, WHITE);

                // Draw Enemy
                let enemy_x = screen_width() / 2.0;
                let enemy_y = screen_height() / 2.0 - 50.0;
                draw_circle(enemy_x, enemy_y, 40.0, RED);
                draw_text(&current_enemy.name, enemy_x - 50.0, enemy_y - 60.0, 30.0, WHITE);

                draw_text(&battle_message, 250.0, screen_height() - 100.0, 30.0, WHITE);

                if player_stats.hp <= 0 {
                    battle_message = "You were defeated... Returning to King.".to_string();
                    if is_key_pressed(KeyCode::Space) {
                        player_stats.hp = player_stats.max_hp;
                        player_pos = vec2(screen_width() / 2.0, screen_height() / 2.0);
                        state = GameState::Field;
                    }
                } else if current_enemy.hp <= 0 {
                    battle_message = format!("You defeated the {}!", current_enemy.name);
                    if is_key_pressed(KeyCode::Space) {
                        state = GameState::Field;
                    }
                } else {
                    if is_player_turn {
                        if !attack_phase {
                            draw_text("Press SPACE to Attack", 250.0, screen_height() - 50.0, 30.0, YELLOW);
                            if is_key_pressed(KeyCode::Space) {
                                attack_phase = true;
                                timing_bar_pos = 0.0;
                                battle_message = "Press SPACE when bar is centered!".to_string();
                            }
                        } else {
                            // Action Command Timing Bar
                            let bar_x = screen_width() / 2.0 - 150.0;
                            let bar_y = enemy_y + 80.0;
                            let bar_width = 300.0;

                            draw_rectangle(bar_x, bar_y, bar_width, 20.0, GRAY);
                            draw_rectangle(bar_x + bar_width / 2.0 - 15.0, bar_y, 30.0, 20.0, GREEN); // Sweet spot

                            timing_bar_pos += timing_bar_dir * 1.5 * dt;
                            if timing_bar_pos >= 1.0 {
                                timing_bar_pos = 1.0;
                                timing_bar_dir = -1.0;
                            } else if timing_bar_pos <= 0.0 {
                                timing_bar_pos = 0.0;
                                timing_bar_dir = 1.0;
                            }

                            let cursor_x = bar_x + timing_bar_pos * bar_width;
                            draw_rectangle(cursor_x - 5.0, bar_y - 5.0, 10.0, 30.0, WHITE);

                            if is_key_pressed(KeyCode::Space) {
                                attack_phase = false;
                                is_player_turn = false;
                                battle_timer = 1.0; // delay for enemy turn

                                // Check sweet spot
                                if timing_bar_pos > 0.45 && timing_bar_pos < 0.55 {
                                    let dmg = player_stats.attack * 2;
                                    current_enemy.hp -= dmg;
                                    battle_message = format!("CRITICAL HIT! {} damage!", dmg);
                                } else if timing_bar_pos > 0.2 && timing_bar_pos < 0.8 {
                                    let dmg = player_stats.attack;
                                    current_enemy.hp -= dmg;
                                    battle_message = format!("Hit! {} damage.", dmg);
                                } else {
                                    battle_message = "Miss!".to_string();
                                }
                            }
                        }
                    } else {
                        // Enemy turn
                        battle_timer -= dt;
                        if battle_timer <= 0.0 {
                            let dmg = current_enemy.attack + rand::gen_range(-2, 3);
                            let dmg = dmg.max(1);
                            player_stats.hp -= dmg;
                            battle_message = format!("{} attacks! {} damage.", current_enemy.name, dmg);
                            is_player_turn = true;
                        }
                    }
                }
            }
        }

        next_frame().await;
    }
}
