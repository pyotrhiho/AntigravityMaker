use game_engine::{AiState, LodLevel, Position, Velocity};
use hecs::{Entity, World};
use macroquad::prelude::*;

// --- Grid Definitions ---
const TILE_SIZE: f32 = 40.0;
const GRID_WIDTH: usize = 20;
const GRID_HEIGHT: usize = 15;

#[derive(Clone, Copy, PartialEq)]
enum RoomType {
    Dirt,
    Empty,
    CommandCenter,
    Barracks,
    Factory,
}

// --- Custom Components ---
struct Worker;
struct Invader;
struct Soldier;

// --- Game Logic ---
pub async fn play_base_game() {
    let mut world = World::new();
    
    // Initialize underground grid (mostly dirt)
    let mut grid = vec![vec![RoomType::Dirt; GRID_WIDTH]; GRID_HEIGHT];
    
    // Make top row surface (Empty)
    for x in 0..GRID_WIDTH {
        grid[0][x] = RoomType::Empty;
    }
    
    // Setup initial Command Center
    let cx = GRID_WIDTH / 2;
    let cy = 2;
    grid[cy][cx] = RoomType::CommandCenter;

    // Spawn an initial worker
    world.spawn((
        Worker,
        Position { x: cx as f32 * TILE_SIZE + TILE_SIZE / 2.0, y: cy as f32 * TILE_SIZE + TILE_SIZE / 2.0 },
        Velocity { x: 0.0, y: 0.0 },
        AiState {
            target: Some(Position { x: (cx + 1) as f32 * TILE_SIZE + TILE_SIZE / 2.0, y: cy as f32 * TILE_SIZE + TILE_SIZE / 2.0 }),
            state: "Idle".to_string(),
        },
        LodLevel::Active,
    ));

    let mut selected_room = RoomType::Empty; // Digging
    let mut invader_spawn_timer = 0.0;
    let mut barracks_spawn_timer = 0.0;
    let mut factory_generate_timer = 0.0;
    let mut resources = 0;

    loop {
        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        let dt = get_frame_time();

        // UI for Selection
        if is_key_pressed(KeyCode::Key1) { selected_room = RoomType::Empty; } // Dig
        if is_key_pressed(KeyCode::Key2) { selected_room = RoomType::Barracks; }
        if is_key_pressed(KeyCode::Key3) { selected_room = RoomType::Factory; }

        // Grid Interaction
        if is_mouse_button_pressed(MouseButton::Left) {
            let mouse_pos = mouse_position();
            let grid_x = (mouse_pos.0 / TILE_SIZE) as usize;
            let grid_y = (mouse_pos.1 / TILE_SIZE) as usize;

            if grid_x < GRID_WIDTH && grid_y < GRID_HEIGHT && grid_y > 0 {
                // Simplified builder logic: just place the room
                // In a real game, workers would travel here and build it over time
                if grid[grid_y][grid_x] == RoomType::Dirt || grid[grid_y][grid_x] == RoomType::Empty {
                    grid[grid_y][grid_x] = selected_room;
                }
            }
        }

        // Spawn Invaders occasionally
        invader_spawn_timer += dt;
        if invader_spawn_timer > 5.0 {
            invader_spawn_timer = 0.0;
            
            // Spawn at surface level
            let ix = rand::gen_range(0, GRID_WIDTH);
            world.spawn((
                Invader,
                Position { x: ix as f32 * TILE_SIZE + TILE_SIZE / 2.0, y: TILE_SIZE / 2.0 },
                Velocity { x: 0.0, y: 0.0 },
                AiState {
                    // Send them straight to the command center
                    target: Some(Position { x: cx as f32 * TILE_SIZE + TILE_SIZE / 2.0, y: cy as f32 * TILE_SIZE + TILE_SIZE / 2.0 }),
                    state: "Invading".to_string(),
                },
                LodLevel::Active,
            ));
        }

        // Give Idle Workers a random target within the dug out base
        for (_, (_pos, ai, _worker)) in world.query_mut::<(&Position, &mut AiState, &Worker)>() {
            if ai.target.is_none() {
                // Find a random non-dirt tile to walk to
                let mut tries = 10;
                while tries > 0 {
                    let rx = rand::gen_range(0, GRID_WIDTH);
                    let ry = rand::gen_range(0, GRID_HEIGHT);
                    if grid[ry][rx] != RoomType::Dirt {
                        ai.target = Some(Position { x: rx as f32 * TILE_SIZE + TILE_SIZE / 2.0, y: ry as f32 * TILE_SIZE + TILE_SIZE / 2.0 });
                        break;
                    }
                    tries -= 1;
                }
            }
        }

        // Logic for Barracks and Factories
        barracks_spawn_timer += dt;
        factory_generate_timer += dt;

        let mut barracks_locations = Vec::new();
        let mut factory_locations = Vec::new();

        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                if grid[y][x] == RoomType::Barracks {
                    barracks_locations.push((x, y));
                } else if grid[y][x] == RoomType::Factory {
                    factory_locations.push((x, y));
                }
            }
        }

        // Factory generates resources over time
        if factory_generate_timer > 3.0 {
            factory_generate_timer = 0.0;
            resources += factory_locations.len() * 10;
        }

        // Barracks spawn Soldiers periodically to defend
        if barracks_spawn_timer > 4.0 {
            barracks_spawn_timer = 0.0;
            for (bx, by) in barracks_locations {
                world.spawn((
                    Soldier,
                    Position { x: bx as f32 * TILE_SIZE + TILE_SIZE / 2.0, y: by as f32 * TILE_SIZE + TILE_SIZE / 2.0 },
                    Velocity { x: 0.0, y: 0.0 },
                    AiState {
                        target: None,
                        state: "Defending".to_string(),
                    },
                    LodLevel::Active,
                ));
            }
        }

        // Give Idle Soldiers a target (seek nearest Invader or patrol)
        // First find an invader
        let mut first_invader_pos = None;
        if let Some((_, (pos, _invader))) = world.query_mut::<(&Position, &Invader)>().into_iter().next() {
            first_invader_pos = Some(*pos);
        }

        for (_, (_pos, ai, _soldier)) in world.query_mut::<(&Position, &mut AiState, &Soldier)>() {
            if let Some(target) = first_invader_pos {
                ai.target = Some(target);
            } else if ai.target.is_none() {
                // Patrol if no invaders
                let mut tries = 10;
                while tries > 0 {
                    let rx = rand::gen_range(0, GRID_WIDTH);
                    let ry = rand::gen_range(0, GRID_HEIGHT);
                    if grid[ry][rx] != RoomType::Dirt {
                        ai.target = Some(Position { x: rx as f32 * TILE_SIZE + TILE_SIZE / 2.0, y: ry as f32 * TILE_SIZE + TILE_SIZE / 2.0 });
                        break;
                    }
                    tries -= 1;
                }
            }
        }

        // Simple Combat: if Soldier touches Invader, both die
        let mut to_despawn = Vec::new();
        let mut invaders = Vec::new();
        for (id, (pos, _invader)) in world.query_mut::<(&Position, &Invader)>() {
            invaders.push((id, *pos));
        }

        let mut soldiers = Vec::new();
        for (id, (pos, _soldier)) in world.query_mut::<(&Position, &Soldier)>() {
            soldiers.push((id, *pos));
        }

        for (inv_id, inv_pos) in invaders {
            for (sol_id, sol_pos) in &soldiers {
                if !to_despawn.contains(sol_id) && !to_despawn.contains(&inv_id) {
                    let dx = inv_pos.x - sol_pos.x;
                    let dy = inv_pos.y - sol_pos.y;
                    if dx * dx + dy * dy < 400.0 { // 20 distance squared
                        to_despawn.push(inv_id);
                        to_despawn.push(*sol_id);
                    }
                }
            }
        }

        for id in to_despawn {
            let _ = world.despawn(id);
        }


        // Run Engine Simulation (Pathfinding approximation)
        game_engine::systems::simulate_ai(&mut world, dt);

        // Render
        clear_background(BLACK);

        // Render Grid
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                let px = x as f32 * TILE_SIZE;
                let py = y as f32 * TILE_SIZE;

                let color = match grid[y][x] {
                    RoomType::Dirt => DARKBROWN,
                    RoomType::Empty => DARKGRAY,
                    RoomType::CommandCenter => BLUE,
                    RoomType::Barracks => RED,
                    RoomType::Factory => ORANGE,
                };
                
                draw_rectangle(px, py, TILE_SIZE - 2.0, TILE_SIZE - 2.0, color);
            }
        }

        // Render Entities
        for (_, (pos, _worker)) in world.query_mut::<(&Position, &Worker)>() {
            draw_circle(pos.x, pos.y, 10.0, WHITE);
        }

        for (_, (pos, _invader)) in world.query_mut::<(&Position, &Invader)>() {
            draw_circle(pos.x, pos.y, 10.0, MAGENTA);
        }

        for (_, (pos, _soldier)) in world.query_mut::<(&Position, &Soldier)>() {
            draw_circle(pos.x, pos.y, 10.0, GREEN);
        }

        // UI
        draw_rectangle(0.0, screen_height() - 60.0, screen_width(), 60.0, LIGHTGRAY);
        draw_text(&format!("Base Builder Demo | Resources: {}", resources), 10.0, screen_height() - 40.0, 20.0, BLACK);
        
        let sel_str = match selected_room {
            RoomType::Empty => "1: Dig",
            RoomType::Barracks => "2: Barracks",
            RoomType::Factory => "3: Factory",
            _ => "",
        };
        draw_text(&format!("Selected: {}", sel_str), 10.0, screen_height() - 15.0, 20.0, DARKBLUE);
        draw_text("Click grid to place. ESC to exit.", 250.0, screen_height() - 15.0, 20.0, BLACK);

        next_frame().await;
    }
}