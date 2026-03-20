use game_engine::{AiState, LodLevel, Position, Velocity, Speed};
use hecs::{Entity, World};
use macroquad::prelude::*;

// --- Custom Components ---
struct Player;
struct Enemy {
    hp: i32,
    hit_cooldown: f32,
}
struct Spear {
    start: Vec2,
    end: Vec2,
    hit_radius: f32,
    damage: i32,
}
struct Lifetime(f32);
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
        Enemy { hp: 2, hit_cooldown: 0.0 },
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

fn handle_input(world: &mut World, player_pos: Vec2, is_jumping: bool) {
    if is_mouse_button_pressed(MouseButton::Left) {
        let mouse_pos = mouse_position();
        let mouse_vec = vec2(mouse_pos.0, mouse_pos.1);
        
        let mut dir = mouse_vec - player_pos;
        if dir.length_squared() > 0.0 {
            dir = dir.normalize();
            // Snap attack direction to nearest 8-way for grid-like feel
            dir = quantize_direction_8way(dir);
        } else {
            dir = vec2(1.0, 0.0);
        }
        
        let spear_length = if is_jumping { 200.0 } else { 150.0 };
        let end_pos = player_pos + dir * spear_length;
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

        if let Ok(mut query) = world.query_one::<(&mut Position, &mut JumpState)>(player_entity) {
            if let Some((pos, jump)) = query.get() {
                let mut move_dir = vec2(0.0, 0.0);

                if jump.is_jumping {
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
            }
        }

        enemy_spawn_timer += dt;
        if enemy_spawn_timer > 0.5 {
            spawn_enemy(&mut world, player_pos, current_enemy_speed);
            enemy_spawn_timer = 0.0;
        }

        handle_input(&mut world, player_pos, is_jumping);

        for (_, (ai, speed, _enemy)) in world.query_mut::<(&mut AiState, &mut Speed, &Enemy)>() {
            ai.target = Some(Position { x: player_pos.x, y: player_pos.y });
            speed.0 = current_enemy_speed; // Also update existing enemies' speed to match elapsed time
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
            }
        }

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

        for (_, (spear,)) in world.query_mut::<(&Spear,)>() {
            draw_line(spear.start.x, spear.start.y, spear.end.x, spear.end.y, 8.0, YELLOW);
        }

        draw_text(&format!("Score: {}", score), 20.0, 40.0, 30.0, WHITE);
        draw_text("WASD to move, Click to Attack", 20.0, 80.0, 20.0, LIGHTGRAY);
        draw_text("Press ESC to return to Menu", 20.0, 110.0, 20.0, ORANGE);

        next_frame().await;
    }
}