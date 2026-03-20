use hecs::World;
use std::time::{Duration, Instant};
use std::thread;

pub struct Engine {
    pub world: World,
    pub tick_rate: u32,
    running: bool,
}

impl Engine {
    pub fn new(tick_rate: u32) -> Self {
        Self {
            world: World::new(),
            tick_rate,
            running: false,
        }
    }

    pub fn start(&mut self) {
        self.running = true;
        let tick_duration = Duration::from_secs_f32(1.0 / self.tick_rate as f32);
        
        println!("Engine started. Running at {} ticks per second.", self.tick_rate);
        
        while self.running {
            let start = Instant::now();
            
            // Execute Systems
            crate::systems::simulate_ai(&mut self.world, tick_duration.as_secs_f32());
            
            let elapsed = start.elapsed();
            if elapsed < tick_duration {
                thread::sleep(tick_duration - elapsed);
            }
        }
    }

    pub fn stop(&mut self) {
        self.running = false;
    }
}