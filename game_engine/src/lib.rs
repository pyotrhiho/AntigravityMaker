pub mod components;
pub mod systems;
pub mod engine;

pub use components::*;
pub use systems::*;
pub use engine::*;

#[cfg(test)]
mod tests {
    use super::*;
    use hecs::World;

    #[test]
    fn test_ai_simulation() {
        let mut world = World::new();
        
        let entity = world.spawn((
            Position { x: 0.0, y: 0.0 },
            Velocity { x: 0.0, y: 0.0 },
            AiState { target: Some(Position { x: 10.0, y: 0.0 }), state: "Moving".to_string() },
            LodLevel::Active,
        ));

        // Simulate 1 second at 60 FPS
        let dt = 1.0 / 60.0;
        for _ in 0..60 {
            simulate_ai(&mut world, dt);
        }

        let mut query = world.query_one::<&Position>(entity).unwrap();
        let pos = query.get().unwrap();
        
        // After 1 second at speed 5.0, x should be close to 5.0
        assert!((pos.x - 5.0).abs() < 0.1);
        assert_eq!(pos.y, 0.0);
    }
}