use hecs::World;
use crate::components::{Position, Velocity, AiState, LodLevel};

pub fn simulate_ai(world: &mut World, dt: f32) {
    for (_id, (pos, vel, ai, lod)) in world.query_mut::<(&mut Position, &mut Velocity, &mut AiState, &LodLevel)>() {
        match lod {
            LodLevel::Active => {
                // Precise pathfinding and collision avoidance for active entities
                if let Some(target) = ai.target {
                    let dx = target.x - pos.x;
                    let dy = target.y - pos.y;
                    let dist = (dx * dx + dy * dy).sqrt();
                    
                    if dist > 1.0 {
                        // Normalize and apply speed
                        vel.x = (dx / dist) * 5.0; // Assume base speed 5.0
                        vel.y = (dy / dist) * 5.0;
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
                        vel.x = (dx / dist) * 5.0;
                        vel.y = (dy / dist) * 5.0;
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