use hecs::World;
use crate::components::{Position, Velocity, AiState, LodLevel, Speed};

pub fn simulate_ai(world: &mut World, dt: f32) {
    // First, let's collect the components we need and default speed if Speed component is not present
    let mut speeds = Vec::new();
    for (id, (_pos, _vel, _ai, _lod)) in world.query_mut::<(&Position, &Velocity, &AiState, &LodLevel)>() {
        speeds.push(id);
    }

    for id in speeds {
        let speed = if let Ok(mut q) = world.query_one::<&Speed>(id) {
            if let Some(s) = q.get() {
                s.0
            } else {
                5.0
            }
        } else {
            5.0 // Default speed
        };

        if let Ok(mut q) = world.query_one::<(&mut Position, &mut Velocity, &mut AiState, &LodLevel)>(id) {
        if let Some((pos, vel, ai, lod)) = q.get() {
        match lod {
            LodLevel::Active => {
                // Precise pathfinding and collision avoidance for active entities
                if let Some(target) = ai.target {
                    let dx = target.x - pos.x;
                    let dy = target.y - pos.y;
                    let dist = (dx * dx + dy * dy).sqrt();
                    
                    if dist > 1.0 {
                        // Normalize and apply speed
                        vel.x = (dx / dist) * speed;
                        vel.y = (dy / dist) * speed;
                    } else {
                        vel.x = 0.0;
                        vel.y = 0.0;
                        ai.target = None; // Reached target
                        ai.state = "Idle".to_string();
                    }
                }
            }
            LodLevel::Simulated => {
                // Simplified simulation: just move towards target directly
                if let Some(target) = ai.target {
                    let dx = target.x - pos.x;
                    let dy = target.y - pos.y;
                    let dist = (dx * dx + dy * dy).sqrt();
                    
                    if dist > 1.0 {
                        vel.x = (dx / dist) * speed;
                        vel.y = (dy / dist) * speed;
                    } else {
                        vel.x = 0.0;
                        vel.y = 0.0;
                        ai.target = None;
                    }
                }
            }
            LodLevel::Background => {
                // Background entities don't need complex pathfinding
                // They either snap to target or don't move at all
                if let Some(target) = ai.target {
                    pos.x = target.x;
                    pos.y = target.y;
                    vel.x = 0.0;
                    vel.y = 0.0;
                    ai.target = None;
                }
            }
        }
        
        // Apply velocity to position
        if *lod != LodLevel::Background {
            pos.x += vel.x * dt;
            pos.y += vel.y * dt;
        }
        }
        }
    }
}