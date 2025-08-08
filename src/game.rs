use crate::types::*;
use crate::collision::*;
use crate::movement::*;
use crate::renderer::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::ttf::Font;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::f32::consts::PI;
use fastrand;

// Main game structure
pub struct Game {
    pub app_state: AppState,
    pub stats: Stats,
    pub start_time: Instant,
    pub cars: Vec<Car>,
    pub next_car_id: usize,
    pub spawn_coords: HashMap<(Route, Direction), (f32, f32)>,
    pub priority_map: HashMap<(usize, usize), usize>,
    pub priority_ref: HashMap<(usize, usize), usize>, //PEAK LOGIC HONESTLY
    pub in_intersection: HashMap<CollisionType, Vec<usize>>,
    pub last_spawn_time: Instant,
    pub spawn_cooldown: Duration,
}

impl Game {
    pub fn new() -> Self {
        let mut spawn_coords = HashMap::new();
        
        spawn_coords.insert((Route::Right, Direction::North), (655.0, 0.0));
        spawn_coords.insert((Route::Right, Direction::West), (1024.0, 435.0));
        spawn_coords.insert((Route::Right, Direction::East), (0.0, 230.0));
        spawn_coords.insert((Route::Right, Direction::South), (360.0, 682.0));
        spawn_coords.insert((Route::Straight, Direction::North), (595.0, 45.0));
        spawn_coords.insert((Route::Straight, Direction::West), (978.0, 390.0));
        spawn_coords.insert((Route::Straight, Direction::East), (0.0, 278.0));
        spawn_coords.insert((Route::Straight, Direction::South), (420.0, 682.0));
        spawn_coords.insert((Route::Left, Direction::North), (535.0, 45.0));
        spawn_coords.insert((Route::Left, Direction::West), (978.0, 350.0));
        spawn_coords.insert((Route::Left, Direction::East), (0.0, 315.0));
        spawn_coords.insert((Route::Left, Direction::South), (480.0, 682.0));
        
        let now = Instant::now();
        Game {
            app_state: AppState::Running,
            stats: Stats::default(),
            start_time: now,
            cars: Vec::new(),
            next_car_id: 1,
            spawn_coords,
            priority_map: HashMap::new(),
            priority_ref: HashMap::new(),
            in_intersection: HashMap::new(),
            last_spawn_time: now,
            spawn_cooldown: Duration::from_millis(0.8), // 0.8 second cooldown between spawns
        }
    }

    pub fn handle_events(&mut self, event_pump: &mut sdl2::EventPump) -> bool {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => return false,
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    match self.app_state {
                        AppState::Running => {
                            self.app_state = AppState::StatsDisplay;
                        }
                        AppState::StatsDisplay => {
                            self.app_state = AppState::Exit;
                        }
                        _ => {}
                    }
                }
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => {
                    if self.app_state == AppState::Running {
                        self.handle_car_spawn_input(keycode);
                    }
                }
                _ => {}
            }
        }
        
        // Check if we should exit
        self.app_state != AppState::Exit
    }

    pub fn update(&mut self, delta_time: f32) {
        match self.app_state {
            AppState::Running => {
                // Update cars with collision detection and route-specific movement
                let car_data: Vec<(usize, f32, f32, Vec<CollisionType>, bool)> = self.cars
                    .iter()
                    .map(|car| (car.id, car.x, car.y, car.collision_types.clone(), car.rotated))
                    .collect();
                
                let car_tracking = build_car_tracking(&car_data);
                
                for car in &mut self.cars {
                    if car.moving {
                        // Check collision before moving
                        if !check_collision(
                            &car_tracking,
                            car.id,
                            car.x,
                            car.y,
                            &car.collision_types,
                            car.rotated,
                            &mut self.priority_map,
                            &mut self.priority_ref,
                            &mut self.stats,
                        ) {
                            // Update speed stats
                            car.max_speed = car.max_speed.max(car.speed);
                            car.min_speed = car.min_speed.min(car.speed);
                            
                            // Update global speed stats
                            self.stats.max_velocity = self.stats.max_velocity.max(car.speed);
                            self.stats.min_velocity = 0.0; // Always 0 since cars stop
                            
                            // Move car based on route
                            match car.route {
                                Route::Straight => move_straight(car, delta_time, &mut self.in_intersection),
                                Route::Right => move_right(car, delta_time, &mut self.in_intersection),
                                Route::Left => move_left(car, delta_time, &mut self.in_intersection),
                            }
                        }
                    }
                }
                
                // Despawn cars that have stopped moving and update stats
                let mut cars_to_remove = Vec::new();
                for (i, car) in self.cars.iter().enumerate() {
                    if !car.moving {
                        // Update stats when car finishes journey
                        let travel_time = car.spawn_time.elapsed();
                        self.stats.max_time = self.stats.max_time.max(travel_time);
                        if travel_time < self.stats.min_time {
                            self.stats.min_time = travel_time;
                        }
                        cars_to_remove.push((i, car.id, car.collision_types[0]));
                    }
                }
                
                // Remove cars in reverse order to maintain indices and clean up intersection
                for &(i, car_id, collision_type) in cars_to_remove.iter().rev() {
                    self.cars.remove(i);
                    
                    // Clean up priority maps
                    self.priority_map.retain(|&(id1, id2), _| id1 != car_id && id2 != car_id);
                    self.priority_ref.retain(|_, owner_id| *owner_id != car_id);
                    
                    // Remove from intersection
                    if let Some(cars_in_intersection) = self.in_intersection.get_mut(&collision_type) {
                        cars_in_intersection.retain(|&x| x != car_id);
                    }
                }
            }
            AppState::StatsDisplay => {
                // Stats display is handled in render
            }
            _ => {}
        }
    }

    pub fn render(&self, canvas: &mut Canvas<Window>, textures: &GameTextures, font: &Font) -> Result<(), String> {
        match self.app_state {
            AppState::Running => {
                render_game(canvas, textures, &self.cars)?;
            }
            AppState::StatsDisplay => {
                render_stats(canvas, font, &self.stats)?;
            }
            _ => {}
        }
        Ok(())
    }
    
    fn handle_car_spawn_input(&mut self, keycode: Keycode) {
        // Check if enough time has passed since last spawn
        let now = Instant::now();
        if now.duration_since(self.last_spawn_time) < self.spawn_cooldown {
            println!("Spawn on cooldown, please wait {:.1} more seconds", 
                (self.spawn_cooldown - now.duration_since(self.last_spawn_time)).as_secs_f32());
            return;
        }
        
        let direction = match keycode {
            Keycode::Up => Some(Direction::North),
            Keycode::Down => Some(Direction::South),
            Keycode::Right => Some(Direction::East),
            Keycode::Left => Some(Direction::West),
            Keycode::R => {
                // Random direction
                let random_dir = fastrand::u32(0..4);
                match random_dir {
                    0 => Some(Direction::North),
                    1 => Some(Direction::South),
                    2 => Some(Direction::East),
                    3 => Some(Direction::West),
                    _ => Some(Direction::North),
                }
            }
            _ => None,
        };
        
        if let Some(dir) = direction {
            // Random route
            let random_route = fastrand::u32(0..3);
            let route = match random_route {
                0 => Route::Right,
                1 => Route::Straight,
                2 => Route::Left,
                _ => Route::Straight,
            };
            
            let speed = if route == Route::Straight { SUPER } else { FAST };
            
            // Get spawn coordinates
            if let Some(&(x, y)) = self.spawn_coords.get(&(route.clone(), dir.clone())) {
                // Determine collision type
                let collision_type = match route {
                    Route::Straight => match dir {
                        Direction::North => CollisionType::NS,
                        Direction::West => CollisionType::WS,
                        Direction::East => CollisionType::ES,
                        Direction::South => CollisionType::SS,
                    },
                    Route::Left => match dir {
                        Direction::North => CollisionType::NL,
                        Direction::West => CollisionType::WL,
                        Direction::East => CollisionType::EL,
                        Direction::South => CollisionType::SL,
                    },
                    _ => CollisionType::GG,
                };
                
                // Collision types list (Basically all possible collision types for the given route and direction)
                let collision_types = match route {
                    Route::Straight => {
                        match dir {
                            Direction::North => vec![CollisionType::NS, CollisionType::WS, CollisionType::ES, CollisionType::WL, CollisionType::SL],
                            Direction::West => vec![CollisionType::WS, CollisionType::SS, CollisionType::NS, CollisionType::EL, CollisionType::SL],
                            Direction::East => vec![CollisionType::ES, CollisionType::SS, CollisionType::NS, CollisionType::WL, CollisionType::NL],
                            Direction::South => vec![CollisionType::SS, CollisionType::WS, CollisionType::ES, CollisionType::EL, CollisionType::NL],
                        }
                    },
                    Route::Left => match dir {
                        Direction::North => vec![CollisionType::NL, CollisionType::ES, CollisionType::SS],
                        Direction::West => vec![CollisionType::WL, CollisionType::NS, CollisionType::ES],
                        Direction::East => vec![CollisionType::EL, CollisionType::SS, CollisionType::WS],
                        Direction::South => vec![CollisionType::SL, CollisionType::NS, CollisionType::ES],
                    },
                    _ => vec![CollisionType::GG],
                };
                
                // Check for spawn collision
                let car_data: Vec<(usize, f32, f32, Vec<CollisionType>, bool)> = self.cars
                    .iter()
                    .map(|car| (car.id, car.x, car.y, car.collision_types.clone(), car.rotated))
                    .collect();
                
                let should_spawn = !check_spawn_collision(x, y, collision_type, &car_data);
                
                if should_spawn {
                    // Initial rotation based on direction
                    let rotation = match dir {
                        Direction::North => 0.0,
                        Direction::East => -PI / 2.0,
                        Direction::South => PI,
                        Direction::West => PI / 2.0,
                    };
                    
                    let car = Car {
                        x,
                        y,
                        speed,
                        direction: dir,
                        route,
                        rotation,
                        id: self.next_car_id,
                        spawn_time: Instant::now(),
                        moving: true,
                        rotated: false,
                        collision_types,
                        max_speed: speed,
                        min_speed: speed,
                        entered: false,
                    };
                    
                    self.cars.push(car);
                    self.next_car_id += 1;
                    self.stats.max_number_cars += 1;
                    self.last_spawn_time = now; // Update last spawn time
                    
                    println!("Spawned car {} at ({}, {})", self.next_car_id - 1, x, y);
                } else {
                    println!("Spawn blocked due to collision.");
                }
            }
        }
    }
}