use game_engine::{AiState, LodLevel, Position, Velocity, Speed};
use hecs::{Entity, World};
use macroquad::prelude::*;

// --- Custom Components ---
struct Player;
struct PlayerState {
    hp: f32,
    max_hp: f32,
    stamina: f32,
    max_stamina: f32,
    stun_timer: f32,
    knockback_vel: Vec2,
}

struct Enemy {
    hp: i32,
    hit_cooldown: f32,
    knockback_power: f32,
}
struct Spear {
    start: Vec2,
    end: Vec2,
    hit_radius: f32,
    damage: i32,
}
struct Lifetime(f32);
struct Hamburger;
struct JumpState {
    is_jumping: bool,
    timer: f32,
    jump_duration: f32,
    momentum: Vec2,
}

// --- Game Logic ---
fn spawn_player(world: &mut World) -> Entity {
    world.spawn((
        Player,
        PlayerState {
            hp: 100.0,
            max_hp: 100.0,
            stamina: 100.0,
            max_stamina: 100.0,
            stun_timer: 0.0,
            knockback_vel: vec2(0.0, 0.0),
        },
        Position {
            x: screen_width() / 2.0,
            y: screen_height() / 2.0,
        },
        JumpState {
            is_jumping: false,
            timer: 0.0,
            jump_duration: 0.5,
            momentum: vec2(0.0, 0.0),
        },
    ))
}

fn spawn_enemy(world: &mut World, player_pos: Vec2, current_enemy_speed: f32) {
    let mut x = 0.0;
    let mut y = 0.0;

    let side = rand::gen_range(0, 4);
    if side == 0 {
        x = rand::gen_range(0.0, screen_width());
        y = -50.0;
    } else if side == 1 {
        x = rand::gen_range(0.0, screen_width());
        y = screen_height() + 50.0;
    } else if side == 2 {
        x = -50.0;
        y = rand::gen_range(0.0, screen_height());
    } else {
        x = screen_width() + 50.0;
        y = rand::gen_range(0.0, screen_height());
    }

    world.spawn((
        Enemy { hp: 2, hit_cooldown: 0.0, knockback_power: 300.0 },
        Position { x, y },
        Velocity { x: 0.0, y: 0.0 },
        Speed(current_enemy_speed),
        AiState {
            target: Some(Position {
                x: player_pos.x,
                y: player_pos.y,
            }),
            state: "Hunting".to_string(),
        },
        LodLevel::Active,
    ));
}

fn quantize_direction_8way(dir: Vec2) -> Vec2 {
    let angle = dir.y.atan2(dir.x);
    // 8-way: snap angle to nearest PI/4
    let pi_over_4 = std::f32::consts::PI / 4.0;
    let snapped_angle = (angle / pi_over_4).round() * pi_over_4;
    vec2(snapped_angle.cos(), snapped_angle.sin())
}

fn handle_input(world: &mut World, player_entity: Entity, player_pos: Vec2, is_jumping: bool) {
    let mut attack_dir = vec2(0.0, 0.0);
    let mut pressed_attack = false;

    if is_key_down(KeyCode::I) { attack_dir.y -= 1.0; pressed_attack |= is_key_pressed(KeyCode::I); }
    if is_key_down(KeyCode::K) { attack_dir.y += 1.0; pressed_attack |= is_key_pressed(KeyCode::K); }
    if is_key_down(KeyCode::J) { attack_dir.x -= 1.0; pressed_attack |= is_key_pressed(KeyCode::J); }
    if is_key_down(KeyCode::L) { attack_dir.x += 1.0; pressed_attack |= is_key_pressed(KeyCode::L); }

    if pressed_attack {
        let mut can_attack = false;
        
        if let Ok(mut query) = world.query_one::<&mut PlayerState>(player_entity) {
            if let Some(p_state) = query.get() {
                if p_state.stamina >= 20.0 {
                    p_state.stamina -= 20.0;
                    can_attack = true;
                }
            }
        }

        if can_attack {
            if attack_dir.length_squared() > 0.0 {
                attack_dir = attack_dir.normalize();
                attack_dir = quantize_direction_8way(attack_dir);
            } else {
                attack_dir = vec2(1.0, 0.0); // default if pressed somehow cancels out
            }

            let spear_length = if is_jumping { 200.0 } else { 150.0 };
            let end_pos = player_pos + attack_dir * spear_length;
            let damage = if is_jumping { 2 } else { 1 };

            world.spawn((
                Spear {
                    start: player_pos,
                    end: end_pos,
                    hit_radius: if is_jumping { 40.0 } else { 20.0 },
                    damage,
                },
                Lifetime(0.2),
            ));
        }
    }
}

fn point_near_segment(p: Vec2, a: Vec2, b: Vec2, radius: f32) -> bool {
    let ab = b - a;
    let ap = p - a;
    
    let len_sq = ab.length_squared();
    if len_sq == 0.0 {
        return (p - a).length() <= radius;
    }

    let t = (ap.dot(ab) / len_sq).clamp(0.0, 1.0);
    let closest = a + ab * t;
    
    (p - closest).length() <= radius
}

pub async fn play_swarm_game() {
    let mut world = World::new();
    let player_entity = spawn_player(&mut world);
    
    let mut enemy_spawn_timer = 0.0;
    let mut score = 0;
    let mut elapsed_time = 0.0;

    loop {
        // Allow exiting back to main menu
        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        let dt = get_frame_time();
        elapsed_time += dt;

        // Base engine speed was 5.0.
        // Instructions: double the initial movement speed in the early stage.
        // Initial speed: 10.0.
        // Also: gradually accelerate enemies over time.
        // Let's add 1.0 to speed every 10 seconds.
        let mut current_enemy_speed = 10.0 + (elapsed_time / 10.0) * 1.0;

        let mut player_pos = vec2(0.0, 0.0);
        let mut is_jumping = false;
        let mut player_hp = 0.0;
        let mut player_max_hp = 0.0;
        let mut player_stamina = 0.0;
        let mut player_max_stamina = 0.0;
        let mut player_is_stunned = false;

        if let Ok(mut query) = world.query_one::<(&mut Position, &mut JumpState, &mut PlayerState)>(player_entity) {
            if let Some((pos, jump, p_state)) = query.get() {
                // Recover stamina
                p_state.stamina += 20.0 * dt;
                if p_state.stamina > p_state.max_stamina {
                    p_state.stamina = p_state.max_stamina;
                }

                let mut move_dir = vec2(0.0, 0.0);

                if p_state.stun_timer > 0.0 {
                    // Stunned: apply knockback and prevent other actions
                    p_state.stun_timer -= dt;
                    pos.x += p_state.knockback_vel.x * dt;
                    pos.y += p_state.knockback_vel.y * dt;
                    p_state.knockback_vel *= 0.9; // decay knockback
                    player_is_stunned = true;
                } else if jump.is_jumping {
                    // In mid-air: move according to inertia, cannot change direction
                    pos.x += jump.momentum.x * 200.0 * dt;
                    pos.y += jump.momentum.y * 200.0 * dt;

                    jump.timer += dt;
                    if jump.timer >= jump.jump_duration {
                        jump.is_jumping = false;
                    }
                } else {
                    if is_key_down(KeyCode::W) { move_dir.y -= 1.0; }
                    if is_key_down(KeyCode::S) { move_dir.y += 1.0; }
                    if is_key_down(KeyCode::A) { move_dir.x -= 1.0; }
                    if is_key_down(KeyCode::D) { move_dir.x += 1.0; }

                    if move_dir.length_squared() > 0.0 {
                        move_dir = move_dir.normalize();
                        move_dir = quantize_direction_8way(move_dir); // Ensure crisp 8-way movement
                        pos.x += move_dir.x * 200.0 * dt;
                        pos.y += move_dir.y * 200.0 * dt;
                    }

                    // Command input buffer/snappy jump
                    if is_key_pressed(KeyCode::Space) {
                        jump.is_jumping = true;
                        jump.timer = 0.0;
                        jump.momentum = move_dir;
                    }
                }

                pos.x = pos.x.clamp(20.0, screen_width() - 20.0);
                pos.y = pos.y.clamp(20.0, screen_height() - 20.0);

                player_pos = vec2(pos.x, pos.y);
                is_jumping = jump.is_jumping;
                player_hp = p_state.hp;
                player_max_hp = p_state.max_hp;
                player_stamina = p_state.stamina;
                player_max_stamina = p_state.max_stamina;
            }
        }

        // Check for game over
        if player_hp <= 0.0 {
            clear_background(BLACK);
            let text = format!("GAME OVER - Score: {}", score);
            let text_dim = measure_text(&text, None, 40, 1.0);
            draw_text(&text, screen_width() / 2.0 - text_dim.width / 2.0, screen_height() / 2.0, 40.0, RED);
            draw_text("Press ESC to return", screen_width() / 2.0 - 100.0, screen_height() / 2.0 + 40.0, 20.0, WHITE);
            next_frame().await;
            continue;
        }

        enemy_spawn_timer += dt;
        if enemy_spawn_timer > 0.5 {
            spawn_enemy(&mut world, player_pos, current_enemy_speed);
            enemy_spawn_timer = 0.0;
        }

        if !player_is_stunned {
            handle_input(&mut world, player_entity, player_pos, is_jumping);
        }

        for (_, (ai, speed, _enemy)) in world.query_mut::<(&mut AiState, &mut Speed, &Enemy)>() {
            ai.target = Some(Position { x: player_pos.x, y: player_pos.y });
            speed.0 = current_enemy_speed; // Also update existing enemies' speed to match elapsed time
        }

        // Enemy collision with player
        if !player_is_stunned {
            let mut hit_knockback = None;
            for (_, (e_pos, enemy)) in world.query_mut::<(&Position, &Enemy)>() {
                let e_vec = vec2(e_pos.x, e_pos.y);
                if (e_vec - player_pos).length() < 35.0 { // Player radius 20 + Enemy radius 15
                    let mut kb_dir = player_pos - e_vec;
                    if kb_dir.length_squared() == 0.0 {
                        kb_dir = vec2(1.0, 0.0);
                    }
                    kb_dir = kb_dir.normalize();
                    hit_knockback = Some(kb_dir * enemy.knockback_power);
                    break; // Take hit from one enemy per frame
                }
            }

            if let Some(kb) = hit_knockback {
                if let Ok(mut query) = world.query_one::<&mut PlayerState>(player_entity) {
                    if let Some(p_state) = query.get() {
                        p_state.hp -= 10.0;
                        p_state.knockback_vel = kb;
                        p_state.stun_timer = 0.3; // 0.3 seconds of stun
                    }
                }
            }
        }

        game_engine::systems::simulate_ai(&mut world, dt);

        let mut to_despawn = Vec::new();
        
        for (id, (lifetime,)) in world.query_mut::<(&mut Lifetime,)>() {
            lifetime.0 -= dt;
            if lifetime.0 <= 0.0 {
                to_despawn.push(id);
            }
        }

        for (_, (enemy,)) in world.query_mut::<(&mut Enemy,)>() {
            if enemy.hit_cooldown > 0.0 {
                enemy.hit_cooldown -= dt;
            }
        }

        let mut spears = Vec::new();
        for (_, (spear,)) in world.query_mut::<(&Spear,)>() {
            spears.push((spear.start, spear.end, spear.hit_radius, spear.damage));
        }

        let mut hamburgers_to_spawn = Vec::new();
        for (enemy_id, (enemy_pos, enemy)) in world.query_mut::<(&Position, &mut Enemy)>() {
            let e_pos = vec2(enemy_pos.x, enemy_pos.y);
            let mut hit_this_frame = false;
            if enemy.hit_cooldown <= 0.0 {
                for (start, end, radius, damage) in &spears {
                    if point_near_segment(e_pos, *start, *end, *radius + 15.0) {
                        hit_this_frame = true;
                        enemy.hp -= damage;
                        enemy.hit_cooldown = 0.5; // Cooldown to avoid multi-hits from the same attack
                        break;
                    }
                }
            }
            if hit_this_frame && enemy.hp <= 0 {
                to_despawn.push(enemy_id);
                score += 1;

                // Drop Hamburger (1% chance)
                if rand::gen_range(0, 100) == 0 {
                    hamburgers_to_spawn.push(Position { x: e_pos.x, y: e_pos.y });
                }
            }
        }

        for pos in hamburgers_to_spawn {
            world.spawn((
                Hamburger,
                pos,
            ));
        }

        let mut hamburgers_to_despawn = Vec::new();
        if !player_is_stunned {
            let mut heal_amount = 0.0;
            for (h_id, (h_pos, _hamburger)) in world.query_mut::<(&Position, &Hamburger)>() {
                let h_vec = vec2(h_pos.x, h_pos.y);
                if (h_vec - player_pos).length() < 30.0 { // Player radius 20 + Hamburger radius 10
                    hamburgers_to_despawn.push(h_id);
                    heal_amount += 10.0; // Heal amount will be 10% of max_hp (which is 100), so 10.0. Calculate per burger.
                }
            }
            if heal_amount > 0.0 {
                if let Ok(mut query) = world.query_one::<&mut PlayerState>(player_entity) {
                    if let Some(p_state) = query.get() {
                        p_state.hp += heal_amount;
                        if p_state.hp > p_state.max_hp {
                            p_state.hp = p_state.max_hp;
                        }
                    }
                }
            }
        }

        to_despawn.extend(hamburgers_to_despawn);
        to_despawn.sort();
        to_despawn.dedup();
        for id in to_despawn {
            let _ = world.despawn(id);
        }

        clear_background(DARKGRAY);

        // draw jumping shadow and player
        if is_jumping {
            draw_circle(player_pos.x, player_pos.y + 20.0, 15.0, BLACK);
            draw_circle(player_pos.x, player_pos.y - 20.0, 20.0, SKYBLUE);
        } else {
            draw_circle(player_pos.x, player_pos.y, 20.0, BLUE);
        }

        for (_, (pos, enemy)) in world.query_mut::<(&Position, &Enemy)>() {
            let color = if enemy.hp == 2 { RED } else { PINK };
            draw_circle(pos.x, pos.y, 15.0, color);
        }

        for (_, (pos, _hamburger)) in world.query_mut::<(&Position, &Hamburger)>() {
            draw_circle(pos.x, pos.y, 10.0, ORANGE);
        }

        for (_, (spear,)) in world.query_mut::<(&Spear,)>() {
            draw_line(spear.start.x, spear.start.y, spear.end.x, spear.end.y, 8.0, YELLOW);
        }

        draw_text(&format!("Score: {}", score), 20.0, 40.0, 30.0, WHITE);

        // Draw HP Bar
        draw_rectangle(20.0, 60.0, 200.0, 20.0, DARKGRAY);
        draw_rectangle(20.0, 60.0, 200.0 * (player_hp / player_max_hp).clamp(0.0, 1.0), 20.0, RED);
        draw_text(&format!("HP: {}/{}", player_hp as i32, player_max_hp as i32), 25.0, 75.0, 20.0, WHITE);

        // Draw Stamina Bar
        draw_rectangle(20.0, 90.0, 200.0, 15.0, DARKGRAY);
        draw_rectangle(20.0, 90.0, 200.0 * (player_stamina / player_max_stamina).clamp(0.0, 1.0), 15.0, GREEN);
        draw_text(&format!("Stamina: {}/{}", player_stamina as i32, player_max_stamina as i32), 25.0, 102.0, 15.0, WHITE);

        draw_text("WASD to move, I/J/K/L to Attack", 20.0, 140.0, 20.0, LIGHTGRAY);
        draw_text("Press ESC to return to Menu", 20.0, 170.0, 20.0, ORANGE);

        next_frame().await;
    }
}