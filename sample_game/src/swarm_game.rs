use game_engine::{AiState, LodLevel, Position, Velocity};
use hecs::{Entity, World};
use macroquad::prelude::*;

// --- Custom Components ---
struct Player;
struct Enemy;
struct Spear {
    start: Vec2,
    end: Vec2,
    hit_radius: f32,
}
struct Lifetime(f32);

// --- Game Logic ---
fn spawn_player(world: &mut World) -> Entity {
    world.spawn((
        Player,
        Position {
            x: screen_width() / 2.0,
            y: screen_height() / 2.0,
        },
    ))
}

fn spawn_enemy(world: &mut World, player_pos: Vec2) {
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
        Enemy,
        Position { x, y },
        Velocity { x: 0.0, y: 0.0 },
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

fn handle_input(world: &mut World, player_pos: Vec2) {
    if is_mouse_button_pressed(MouseButton::Left) {
        let mouse_pos = mouse_position();
        let mouse_vec = vec2(mouse_pos.0, mouse_pos.1);
        
        let mut dir = mouse_vec - player_pos;
        if dir.length_squared() > 0.0 {
            dir = dir.normalize();
        } else {
            dir = vec2(1.0, 0.0);
        }
        
        let spear_length = 150.0;
        let end_pos = player_pos + dir * spear_length;

        world.spawn((
            Spear {
                start: player_pos,
                end: end_pos,
                hit_radius: 20.0,
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

    loop {
        // Allow exiting back to main menu
        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        let dt = get_frame_time();

        let mut player_pos = vec2(0.0, 0.0);
        if let Ok(mut pos) = world.query_one::<&mut Position>(player_entity) {
            if let Some(pos) = pos.get() {
                let mut move_dir = vec2(0.0, 0.0);
                if is_key_down(KeyCode::W) { move_dir.y -= 1.0; }
                if is_key_down(KeyCode::S) { move_dir.y += 1.0; }
                if is_key_down(KeyCode::A) { move_dir.x -= 1.0; }
                if is_key_down(KeyCode::D) { move_dir.x += 1.0; }
                
                if move_dir.length_squared() > 0.0 {
                    move_dir = move_dir.normalize();
                    pos.x += move_dir.x * 200.0 * dt;
                    pos.y += move_dir.y * 200.0 * dt;
                }

                pos.x = pos.x.clamp(20.0, screen_width() - 20.0);
                pos.y = pos.y.clamp(20.0, screen_height() - 20.0);

                player_pos = vec2(pos.x, pos.y);
            }
        }

        enemy_spawn_timer += dt;
        if enemy_spawn_timer > 0.5 {
            spawn_enemy(&mut world, player_pos);
            enemy_spawn_timer = 0.0;
        }

        handle_input(&mut world, player_pos);

        for (_, (ai, _enemy)) in world.query_mut::<(&mut AiState, &Enemy)>() {
            ai.target = Some(Position { x: player_pos.x, y: player_pos.y });
        }

        game_engine::systems::simulate_ai(&mut world, dt);

        let mut to_despawn = Vec::new();
        
        for (id, (lifetime,)) in world.query_mut::<(&mut Lifetime,)>() {
            lifetime.0 -= dt;
            if lifetime.0 <= 0.0 {
                to_despawn.push(id);
            }
        }

        let mut spears = Vec::new();
        for (_, (spear,)) in world.query_mut::<(&Spear,)>() {
            spears.push((spear.start, spear.end, spear.hit_radius));
        }

        for (enemy_id, (enemy_pos, _enemy)) in world.query_mut::<(&Position, &Enemy)>() {
            let e_pos = vec2(enemy_pos.x, enemy_pos.y);
            for (start, end, radius) in &spears {
                if point_near_segment(e_pos, *start, *end, *radius + 15.0) {
                    to_despawn.push(enemy_id);
                    score += 1;
                    break;
                }
            }
        }

        to_despawn.sort();
        to_despawn.dedup();
        for id in to_despawn {
            let _ = world.despawn(id);
        }

        clear_background(DARKGRAY);

        draw_circle(player_pos.x, player_pos.y, 20.0, BLUE);

        for (_, (pos, _enemy)) in world.query_mut::<(&Position, &Enemy)>() {
            draw_circle(pos.x, pos.y, 15.0, RED);
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