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
        for (_, (pos, ai, _worker)) in world.query_mut::<(&Position, &mut AiState, &Worker)>() {
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

        // UI
        draw_rectangle(0.0, screen_height() - 60.0, screen_width(), 60.0, LIGHTGRAY);
        draw_text("Base Builder Demo", 10.0, screen_height() - 40.0, 20.0, BLACK);
        
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