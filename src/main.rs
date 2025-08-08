use sdl2::event::Event;
use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::{Canvas, Texture, TextureCreator, BlendMode};
use sdl2::video::{Window, WindowContext};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use fastrand;
use std::f32::consts::PI;

const WINDOW_WIDTH: u32 = 1024;
const WINDOW_HEIGHT: u32 = 682;

// Game state and app states
#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    Running,
    Paused,
    StatsDisplay,
    Exit,
}

// Speed constants
pub const SLOW: f32 = 50.0;
pub const MEDIUM: f32 = 100.0;
pub const FAST: f32 = 150.0;
pub const SUPER: f32 = 200.0;

// Car directions and routes
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Route {
    Right,
    Straight,
    Left,
}

// Collision types from Bevy version
#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub enum CollisionType {
    NS, // North-South straight
    WS, // West-South straight  
    ES, // East-South straight
    SS, // South-South straight
    NL, // North-Left
    WL, // West-Left
    EL, // East-Left  
    SL, // South-Left
    GG, // Generic/Right turns
}

// 2D Vector for collision detection
#[derive(Debug, Clone, Copy)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Vec2 { x, y }
    }
    
    pub fn dot(self, other: Vec2) -> f32 {
        self.x * other.x + self.y * other.y
    }
    
    pub fn normalize(self) -> Vec2 {
        let len = (self.x * self.x + self.y * self.y).sqrt();
        if len > 0.0 {
            Vec2::new(self.x / len, self.y / len)
        } else {
            Vec2::new(0.0, 0.0)
        }
    }
}

// Car structure
#[derive(Debug, Clone)]
pub struct Car {
    pub x: f32,
    pub y: f32,
    pub speed: f32,
    pub direction: Direction,
    pub route: Route,
    pub rotation: f32, // in radians
    pub id: usize,
    pub spawn_time: Instant,
    pub moving: bool,
    pub rotated: bool, // for tracking if car has rotated during turn
    pub collision_types: Vec<CollisionType>,
    pub max_speed: f32,
    pub min_speed: f32,
    pub entered: bool, // has entered the intersection
}

// Game statistics
#[derive(Debug)]
pub struct Stats {
    pub max_number_cars: usize,
    pub max_velocity: f32,
    pub min_velocity: f32,
    pub max_time: Duration,
    pub min_time: Duration,
    pub close_call: usize,
}

impl Default for Stats {
    fn default() -> Self {
        Stats {
            max_number_cars: 0,
            max_velocity: 0.0,
            min_velocity: 0.0,
            max_time: Duration::from_secs(0),
            min_time: Duration::from_secs(1000),
            close_call: 0,
        }
    }
}

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
        
        Game {
            app_state: AppState::Running,
            stats: Stats::default(),
            start_time: Instant::now(),
            cars: Vec::new(),
            next_car_id: 1,
            spawn_coords,
            priority_map: HashMap::new(),
            priority_ref: HashMap::new(),
            in_intersection: HashMap::new(),
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
                
                let car_tracking = Self::build_car_tracking(&car_data);
                
                for car in &mut self.cars {
                    if car.moving {
                        // Check collision before moving
                        if !Self::check_collision(
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
                            
                            // Move car based on route
                            match car.route {
                                Route::Straight => Self::move_straight(car, delta_time, &mut self.in_intersection),
                                Route::Right => Self::move_right(car, delta_time, &mut self.in_intersection),
                                Route::Left => Self::move_left(car, delta_time, &mut self.in_intersection),
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

    pub fn render(&self, canvas: &mut Canvas<Window>, textures: &GameTextures) -> Result<(), String> {
        match self.app_state {
            AppState::Running => {
                self.render_game(canvas, textures)?;
            }
            AppState::StatsDisplay => {
                self.render_stats(canvas)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn render_game(&self, canvas: &mut Canvas<Window>, textures: &GameTextures) -> Result<(), String> {
        // Create a render target texture for off-screen rendering
        let texture_creator = canvas.texture_creator();
        let mut target_texture = texture_creator.create_texture_target(
            canvas.default_pixel_format(),
            WINDOW_WIDTH,
            WINDOW_HEIGHT
        ).map_err(|e| e.to_string())?;
        
        // Set up the target texture for rendering
        target_texture.set_blend_mode(BlendMode::Blend);
        
        // Render the entire scene to the off-screen texture
        canvas.with_texture_canvas(&mut target_texture, |texture_canvas| {
            // Clear with black background
            texture_canvas.set_draw_color(Color::RGB(0, 0, 0));
            texture_canvas.clear();

            // Draw intersection background
            let dst_rect = sdl2::rect::Rect::new(0, 0, WINDOW_WIDTH, WINDOW_HEIGHT);
            if let Err(e) = texture_canvas.copy(&textures.background, None, Some(dst_rect)) {
                println!("Background copy error: {}", e);
            }

            // Draw cars with rotation
            for car in &self.cars {
                let car_width = 32;  // Reduced from 46
                let car_height = 60; // Reduced from 90
                let dst_rect = sdl2::rect::Rect::new(
                    (car.x - car_width as f32 / 2.0) as i32,
                    (car.y - car_height as f32 / 2.0) as i32,
                    car_width,
                    car_height,
                );
                
                // Convert rotation from radians to degrees for SDL2
                let angle_degrees = car.rotation * 180.0 / PI;
                
                // Draw with rotation
                if let Err(e) = texture_canvas.copy_ex(
                    &textures.car,
                    None,
                    Some(dst_rect),
                    angle_degrees as f64,
                    None,
                    false,
                    false,
                ) {
                    println!("Car copy error: {}", e);
                }
            }
        }).map_err(|e| e.to_string())?;
        
        // Clear the main canvas
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        
        // Copy the off-screen texture to the main canvas with horizontal mirroring
        canvas.copy_ex(
            &target_texture,
            None,
            None,
            0.0,
            None,
            false,
            true,
        )?;

        canvas.present();
        Ok(())
    }

    fn render_stats(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        // Clear with black background
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        // TODO: Implement stats display with text rendering
        // For now, just show black screen with ESC instruction
        
        canvas.present();
        Ok(())
    }
    
    fn handle_car_spawn_input(&mut self, keycode: Keycode) {
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
                
                // Build collision types list (from Bevy version)
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
                
                let should_spawn = !Self::check_spawn_collision(x, y, collision_type, &car_data);
                
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
                    
                    println!("Spawned car {} at ({}, {})", self.next_car_id - 1, x, y);
                } else {
                    println!("Spawn blocked due to collision.");
                }
            }
        }
    }
    
    // Movement functions based on Bevy version with intersection management
    fn move_straight(car: &mut Car, delta_time: f32, in_intersection: &mut HashMap<CollisionType, Vec<usize>>) {
        match car.direction {
            Direction::North => {
                if car.y < 682.0 {
                    let mut counter = 0;
                    for cars_list in in_intersection.values() {
                        if !cars_list.is_empty() {
                            counter += 1;
                        }
                    }
                    let ns_cars = in_intersection.entry(CollisionType::NS).or_insert(Vec::new());
                    if !ns_cars.is_empty() {
                        counter -= 1;
                    }
                    
                    if !car.entered || counter < 3 {
                        car.y += car.speed * delta_time;
                    }
                    if car.y > 170.0 {
                        let ns_cars = in_intersection.entry(CollisionType::NS).or_insert(Vec::new());
                        if !ns_cars.contains(&car.id) {
                            car.entered = true;
                            if counter < 3 {
                                ns_cars.push(car.id);
                            }
                        }
                    }
                } else {
                    car.moving = false;
                }
            }
            Direction::South => {
                if car.y > 0.0 {
                    let mut counter = 0;
                    for cars_list in in_intersection.values() {
                        if !cars_list.is_empty() {
                            counter += 1;
                        }
                    }
                    let ss_cars = in_intersection.entry(CollisionType::SS).or_insert(Vec::new());
                    if !ss_cars.is_empty() {
                        counter -= 1;
                    }
                    
                    if !car.entered || counter < 3 {
                        car.y -= car.speed * delta_time;
                    }
                    if car.y < 540.0 {
                        let ss_cars = in_intersection.entry(CollisionType::SS).or_insert(Vec::new());
                        if !ss_cars.contains(&car.id) {
                            car.entered = true;
                            if counter < 3 {
                                ss_cars.push(car.id);
                            }
                        }
                    }
                } else {
                    car.moving = false;
                }
            }
            Direction::East => {
                if car.x < 1023.0 {
                    let mut counter = 0;
                    for cars_list in in_intersection.values() {
                        if !cars_list.is_empty() {
                            counter += 1;
                        }
                    }
                    let es_cars = in_intersection.entry(CollisionType::ES).or_insert(Vec::new());
                    if !es_cars.is_empty() {
                        counter -= 1;
                    }
                    
                    if !car.entered || counter < 3 {
                        car.x += car.speed * delta_time;
                    }
                    if car.x > 270.0 {
                        let es_cars = in_intersection.entry(CollisionType::ES).or_insert(Vec::new());
                        if !es_cars.contains(&car.id) {
                            car.entered = true;
                            if counter < 3 {
                                es_cars.push(car.id);
                            }
                        }
                    }
                } else {
                    car.moving = false;
                }
            }
            Direction::West => {
                if car.x > 0.0 {
                    let mut counter = 0;
                    for cars_list in in_intersection.values() {
                        if !cars_list.is_empty() {
                            counter += 1;
                        }
                    }
                    let ws_cars = in_intersection.entry(CollisionType::WS).or_insert(Vec::new());
                    if !ws_cars.is_empty() {
                        counter -= 1;
                    }
                    
                    if !car.entered || counter < 3 {
                        car.x -= car.speed * delta_time;
                    }
                    if car.x < 760.0 {
                        let ws_cars = in_intersection.entry(CollisionType::WS).or_insert(Vec::new());
                        if !ws_cars.contains(&car.id) {
                            car.entered = true;
                            if counter < 3 {
                                ws_cars.push(car.id);
                            }
                        }
                    }
                } else {
                    car.moving = false;
                }
            }
        }
    }
    
    fn move_right(car: &mut Car, delta_time: f32, _in_intersection: &mut HashMap<CollisionType, Vec<usize>>) {
        match car.direction {
            Direction::North => {
                if car.y < 230.0 {
                    car.y += car.speed * delta_time;
                    car.speed -= 0.2;
                    if car.y > 180.0 {
                        car.x += 0.3;
                        if !car.rotated {
                            car.rotation = -PI / 2.0;
                            car.rotated = true;
                        }
                    }
                } else if car.x < 1200.0 {
                    car.speed += 2.0;
                    car.x += car.speed * delta_time;
                } else {
                    car.moving = false;
                }
            }
            Direction::West => {
                if car.x > 655.0 {
                    car.x -= car.speed * delta_time;
                    car.speed -= 0.1;
                    if car.x < 730.0 {
                        car.y += 0.3;
                        if !car.rotated {
                            car.rotation = 0.0;
                            car.rotated = true;
                        }
                    }
                } else if car.y < 720.0 {
                    car.speed += 2.0;
                    car.y += car.speed * delta_time;
                } else {
                    car.moving = false;
                }
            }
            Direction::South => {
                if car.y > 433.0 {
                    car.y -= car.speed * delta_time;
                    car.speed -= 0.2;
                    if car.y < 540.0 {
                        car.x -= 0.2;
                        if !car.rotated {
                            car.rotation = PI / 2.0;
                            car.rotated = true;
                        }
                    }
                } else if car.x > -50.0 {
                    car.speed += 2.0;
                    car.x -= car.speed * delta_time;
                } else {
                    car.moving = false;
                }
            }
            Direction::East => {
                if car.x < 360.0 {
                    car.x += car.speed * delta_time;
                    car.speed -= 0.1;
                    if car.x > 285.0 {
                        car.y -= 0.3;
                        if !car.rotated {
                            car.rotation = PI;
                            car.rotated = true;
                        }
                    }
                } else if car.y > -50.0 {
                    car.speed += 2.0;
                    car.y -= car.speed * delta_time;
                } else {
                    car.moving = false;
                }
            }
        }
    }
    
    fn move_left(car: &mut Car, delta_time: f32, in_intersection: &mut HashMap<CollisionType, Vec<usize>>) {
        match car.direction {
            Direction::North => {
                if car.y < 350.0 {
                    let mut counter = 0;
                    let mut counter_left = 0;
                    for cars_list in in_intersection.values() {
                        if !cars_list.is_empty() {
                            counter += 1;
                        }
                    }
                    if in_intersection.entry(CollisionType::WL).or_insert(Vec::new()).len() > 0 ||
                       in_intersection.entry(CollisionType::SL).or_insert(Vec::new()).len() > 0 ||
                       in_intersection.entry(CollisionType::EL).or_insert(Vec::new()).len() > 0 {
                        counter_left += 1;
                    }
                    let nl_cars = in_intersection.entry(CollisionType::NL).or_insert(Vec::new());
                    if !nl_cars.is_empty() {
                        counter -= 1;
                    }
                    
                    if !car.entered || (counter < 3 && counter_left < 1) {
                        car.y += car.speed * delta_time;
                    }
                    if car.y > 307.0 {
                        if !car.rotated {
                            car.rotation = -PI / 2.0;
                            car.rotated = true;
                        }
                    }
                    if car.y > 170.0 {
                        let nl_cars = in_intersection.entry(CollisionType::NL).or_insert(Vec::new());
                        if !nl_cars.contains(&car.id) {
                            car.entered = true;
                            if counter < 3 && counter_left < 1 {
                                nl_cars.push(car.id);
                            }
                        }
                    }
                } else if car.x > 0.0 {
                    car.rotated = true;
                    car.speed += 2.0;
                    car.x -= car.speed * delta_time;
                } else {
                    car.moving = false;
                }
            }
            Direction::South => {
                if car.y > 315.0 {
                    let mut counter = 0;
                    let mut counter_left = 0;
                    for cars_list in in_intersection.values() {
                        if !cars_list.is_empty() {
                            counter += 1;
                        }
                    }
                    if in_intersection.entry(CollisionType::WL).or_insert(Vec::new()).len() > 0 ||
                       in_intersection.entry(CollisionType::NL).or_insert(Vec::new()).len() > 0 ||
                       in_intersection.entry(CollisionType::EL).or_insert(Vec::new()).len() > 0 {
                        counter_left += 1;
                    }
                    let sl_cars = in_intersection.entry(CollisionType::SL).or_insert(Vec::new());
                    if !sl_cars.is_empty() {
                        counter -= 1;
                    }
                    
                    if !car.entered || (counter < 3 && counter_left < 1) {
                        car.y -= car.speed * delta_time;
                    }
                    if car.y < 388.0 {
                        if !car.rotated {
                            car.rotation = PI / 2.0;
                            car.rotated = true;
                        }
                    }
                    if car.y < 540.0 {
                        let sl_cars = in_intersection.entry(CollisionType::SL).or_insert(Vec::new());
                        if !sl_cars.contains(&car.id) {
                            car.entered = true;
                            if counter < 3 && counter_left < 1 {
                                sl_cars.push(car.id);
                            }
                        }
                    }
                } else if car.x < 1200.0 {
                    car.rotated = true;
                    car.speed += 2.0;
                    car.x += car.speed * delta_time;
                } else {
                    car.moving = false;
                }
            }
            Direction::East => {
                if car.x < 538.0 {
                    let mut counter = 0;
                    let mut counter_left = 0;
                    for cars_list in in_intersection.values() {
                        if !cars_list.is_empty() {
                            counter += 1;
                        }
                    }
                    if in_intersection.entry(CollisionType::WL).or_insert(Vec::new()).len() > 0 ||
                       in_intersection.entry(CollisionType::SL).or_insert(Vec::new()).len() > 0 ||
                       in_intersection.entry(CollisionType::NL).or_insert(Vec::new()).len() > 0 {
                        counter_left += 1;
                    }
                    let el_cars = in_intersection.entry(CollisionType::EL).or_insert(Vec::new());
                    if !el_cars.is_empty() {
                        counter -= 1;
                    }
                    
                    if !car.entered || (counter < 3 && counter_left < 1) {
                        car.x += car.speed * delta_time;
                    }
                    if car.x > 490.0 {
                        if !car.rotated {
                            car.rotation = 0.0;
                            car.rotated = true;
                        }
                    }
                    if car.x > 273.0 {
                        let el_cars = in_intersection.entry(CollisionType::EL).or_insert(Vec::new());
                        if !el_cars.contains(&car.id) {
                            car.entered = true;
                            if counter < 3 && counter_left < 1 {
                                el_cars.push(car.id);
                            }
                        }
                    }
                } else if car.y < 700.0 {
                    car.speed += 2.0;
                    car.rotated = true;
                    car.y += car.speed * delta_time;
                } else {
                    car.moving = false;
                }
            }
            Direction::West => {
                if car.x > 477.0 {
                    let mut counter = 0;
                    let mut counter_left = 0;
                    for cars_list in in_intersection.values() {
                        if !cars_list.is_empty() {
                            counter += 1;
                        }
                    }
                    if in_intersection.entry(CollisionType::EL).or_insert(Vec::new()).len() > 0 ||
                       in_intersection.entry(CollisionType::SL).or_insert(Vec::new()).len() > 0 ||
                       in_intersection.entry(CollisionType::NL).or_insert(Vec::new()).len() > 0 {
                        counter_left += 1;
                    }
                    let wl_cars = in_intersection.entry(CollisionType::WL).or_insert(Vec::new());
                    if !wl_cars.is_empty() {
                        counter -= 1;
                    }
                    
                    if !car.entered || (counter < 3 && counter_left < 1) {
                        car.x -= car.speed * delta_time;
                    }
                    if car.x < 535.0 {
                        if !car.rotated {
                            car.rotation = PI;
                            car.rotated = true;
                        }
                    }
                    if car.x < 740.0 {
                        let wl_cars = in_intersection.entry(CollisionType::WL).or_insert(Vec::new());
                        if !wl_cars.contains(&car.id) {
                            car.entered = true;
                            if counter < 3 && counter_left < 1 {
                                wl_cars.push(car.id);
                            }
                        }
                    }
                } else if car.y > 0.0 {
                    car.speed += 2.0;
                    car.rotated = true;
                    car.y -= car.speed * delta_time;
                } else {
                    car.moving = false;
                }
            }
        }
    }
    
    // Collision detection functions from Bevy version
    const HITBOX_BUFFER: f32 = 2.0;
    
    fn build_car_tracking(car_data: &[(usize, f32, f32, Vec<CollisionType>, bool)]) -> HashMap<usize, Vec<(f32, f32, usize, CollisionType, bool)>> {
        let mut car_tracking: HashMap<usize, Vec<(f32, f32, usize, CollisionType, bool)>> = HashMap::new();
        
        for (id, x, y, types, rotated) in car_data {
            let mut temp_cor_car = Vec::new();
            for (other_id, ox, oy, other_types, did_rotate) in car_data {
                if id == other_id {
                    continue;
                }
                let filtered_types: Vec<CollisionType> = types.iter()
                    .cloned()
                    .filter(|t| *t != CollisionType::GG)
                    .collect();
                let filtered_other_types: Vec<CollisionType> = other_types.iter()
                    .cloned()
                    .filter(|t| *t != CollisionType::GG)
                    .collect();
                    
                if filtered_types.iter().any(|t| filtered_other_types.contains(t)) {
                    if !filtered_other_types.is_empty() {
                        temp_cor_car.push((*ox, *oy, *other_id, filtered_other_types[0], *did_rotate));
                    }
                }
            }
            car_tracking.insert(*id, temp_cor_car);
        }
        car_tracking
    }
    
    fn check_collision(
        car_tracking: &HashMap<usize, Vec<(f32, f32, usize, CollisionType, bool)>>,
        car_id: usize,
        car_x: f32,
        car_y: f32,
        collision_types: &[CollisionType],
        rotated: bool,
        priority_map: &mut HashMap<(usize, usize), usize>,
        priority_ref: &mut HashMap<(usize, usize), usize>,
        stats: &mut Stats,
    ) -> bool {
        let primary_type = collision_types.get(0).copied().unwrap_or(CollisionType::GG);
        let car_corners = Self::compute_rotated_corners(car_x, car_y, primary_type, rotated);
        
        let mut temp_win = 0;
        
        if let Some(others) = car_tracking.get(&car_id) {
            for &(x, y, other_id, other_type, did_rotate) in others {
                let other_corners = Self::compute_rotated_corners(x, y, other_type, did_rotate);
                
                if Self::sat_collision(&car_corners, &other_corners) {
                    let pair = (car_id.min(other_id), car_id.max(other_id));
                    
                    // Check existing winner from priority_map
                    if let Some(&winner) = priority_map.get(&pair) {
                        temp_win = winner;
                        if winner != car_id {
                            stats.close_call += 1;
                            return true;
                        }
                    }
                    
                    // Determine reference point based on collision types
                    let (ref_x, ref_y) = Self::get_reference_point(primary_type, other_type);
                    
                    let key = (ref_x as usize, ref_y as usize);
                    // Check if either car has already reached the reference point
                    if let Some(&owner) = priority_ref.get(&key) {
                        if owner == other_id {
                            return true;
                        } else if owner == car_id {
                            continue;
                        }
                    }
                    
                    // Fallback to ID comparison for generic collisions
                    if ref_x == 500.0 {
                        if car_id > other_id {
                            return true;
                        }
                    }
                    
                    // Compare distances to reference point
                    let this_distance = ((car_x - ref_x).powi(2) + (car_y - ref_y).powi(2)).sqrt();
                    let other_distance = ((x - ref_x).powi(2) + (y - ref_y).powi(2)).sqrt();
                    
                    if temp_win == 0 && ref_x != 500.0 {
                        if this_distance > other_distance {
                            if primary_type != other_type {
                                stats.close_call += 1;
                                priority_map.insert(pair, other_id);
                                return true;
                            }
                        } else if primary_type != other_type {
                            priority_map.insert(pair, car_id);
                        }
                    }
                }
            }
        }
        
        // Update reference points
        Self::update_reference_points(car_x, car_y, &car_corners, car_id, priority_ref);
        
        false
    }
    
    fn check_spawn_collision(
        x: f32,
        y: f32,
        collision_type: CollisionType,
        car_data: &[(usize, f32, f32, Vec<CollisionType>, bool)],
    ) -> bool {
        let car_corners = Self::compute_rotated_corners(x, y, collision_type, false);
        
        for (_, ox, oy, other_types, did_rotate) in car_data {
            if let Some(&other_type) = other_types.get(0) {
                if collision_type == other_type {
                    let other_corners = Self::compute_rotated_corners(*ox, *oy, other_type, *did_rotate);
                    if Self::sat_collision(&car_corners, &other_corners) {
                        return true;
                    }
                }
            }
        }
        false
    }
    
    fn get_reference_point(type1: CollisionType, type2: CollisionType) -> (f32, f32) {
        match (type1, type2) {
            (CollisionType::NS, CollisionType::ES) | (CollisionType::ES, CollisionType::NS) => (600.0, 292.0),
            (CollisionType::NS, CollisionType::WS) | (CollisionType::WS, CollisionType::NS) => (600.0, 410.0),
            (CollisionType::WS, CollisionType::SS) | (CollisionType::SS, CollisionType::WS) => (415.0, 410.0),
            (CollisionType::SS, CollisionType::ES) | (CollisionType::ES, CollisionType::SS) => (415.0, 292.0),
            (CollisionType::NL, CollisionType::SS) | (CollisionType::SS, CollisionType::NL) => (415.0, 365.0),
            (CollisionType::NL, CollisionType::ES) | (CollisionType::ES, CollisionType::NL) => (535.0, 292.0),
            (CollisionType::SL, CollisionType::WS) | (CollisionType::WS, CollisionType::SL) => (480.0, 410.0),
            (CollisionType::SL, CollisionType::NS) | (CollisionType::NS, CollisionType::SL) => (600.0, 330.0),
            (CollisionType::EL, CollisionType::SS) | (CollisionType::SS, CollisionType::EL) => (415.0, 330.0),
            (CollisionType::EL, CollisionType::WS) | (CollisionType::WS, CollisionType::EL) => (540.0, 415.0),
            (CollisionType::WL, CollisionType::NS) | (CollisionType::NS, CollisionType::WL) => (600.0, 365.0),
            (CollisionType::WL, CollisionType::ES) | (CollisionType::ES, CollisionType::WL) => (480.0, 292.0),
            _ => (500.0, 500.0), // fallback
        }
    }
    
    fn update_reference_points(
        car_x: f32,
        car_y: f32,
        car_corners: &[Vec2; 4],
        car_id: usize,
        priority_ref: &mut HashMap<(usize, usize), usize>,
    ) {
        let reference_points = [
            (600.0, 292.0), (600.0, 410.0), (415.0, 410.0), (415.0, 292.0),
            (415.0, 365.0), (415.0, 292.0), (535.0, 292.0), (480.0, 410.0),
            (600.0, 330.0), (415.0, 330.0), (540.0, 415.0), (600.0, 365.0),
            (480.0, 292.0),
        ];
        
        for &(ref_x, ref_y) in &reference_points {
            if Self::contains_point(car_corners, ref_x, ref_y) {
                let key = (ref_x.round() as usize, ref_y.round() as usize);
                if !priority_ref.contains_key(&key) {
                    priority_ref.insert(key, car_id);
                }
            }
        }
    }
    
    fn compute_rotated_corners(x: f32, y: f32, collision_type: CollisionType, rotated: bool) -> [Vec2; 4] {
        let (mut width, mut height) = match collision_type {
            CollisionType::NS | CollisionType::SS | CollisionType::NL | CollisionType::SL => (24.0, 100.0),
            CollisionType::ES | CollisionType::WS | CollisionType::WL | CollisionType::EL => (100.0, 24.0),
            _ => (100.0, 100.0),
        };
        
        if rotated {
            std::mem::swap(&mut width, &mut height);
        }
        
        let hw = width / 2.0 + Self::HITBOX_BUFFER;
        let hh = height / 2.0 + Self::HITBOX_BUFFER;
        
        [
            Vec2::new(x - hw, y - hh), // Bottom-left
            Vec2::new(x + hw, y - hh), // Bottom-right
            Vec2::new(x + hw, y + hh), // Top-right
            Vec2::new(x - hw, y + hh), // Top-left
        ]
    }
    
    fn sat_collision(a: &[Vec2; 4], b: &[Vec2; 4]) -> bool {
        let mut axes = Vec::with_capacity(8);
        
        // Get normals of each edge as potential separating axes
        for i in 0..4 {
            let p1 = a[i];
            let p2 = a[(i + 1) % 4];
            let edge = Vec2::new(p2.x - p1.x, p2.y - p1.y);
            let normal = Vec2::new(-edge.y, edge.x).normalize();
            axes.push(normal);
        }
        
        for i in 0..4 {
            let p1 = b[i];
            let p2 = b[(i + 1) % 4];
            let edge = Vec2::new(p2.x - p1.x, p2.y - p1.y);
            let normal = Vec2::new(-edge.y, edge.x).normalize();
            axes.push(normal);
        }
        
        for axis in axes {
            let (a_min, a_max) = Self::project(a, axis);
            let (b_min, b_max) = Self::project(b, axis);
            
            if a_max < b_min || b_max < a_min {
                return false; // Separating axis found
            }
        }
        
        true // Overlap on all axes → collision
    }
    
    fn project(points: &[Vec2; 4], axis: Vec2) -> (f32, f32) {
        let mut min = axis.dot(points[0]);
        let mut max = min;
        
        for point in points.iter().skip(1) {
            let val = axis.dot(*point);
            if val < min { min = val; }
            if val > max { max = val; }
        }
        
        (min, max)
    }
    
    fn contains_point(corners: &[Vec2; 4], px: f32, py: f32) -> bool {
        let mut inside = false;
        for i in 0..4 {
            let j = (i + 1) % 4;
            let xi = corners[i].x;
            let yi = corners[i].y;
            let xj = corners[j].x;
            let yj = corners[j].y;
            
            if ((yi > py) != (yj > py)) && (px < (xj - xi) * (py - yi) / (yj - yi) + xi) {
                inside = !inside;
            }
        }
        inside
    }
}

// Texture management
pub struct GameTextures<'a> {
    pub background: Texture<'a>,
    pub car: Texture<'a>,
}

impl<'a> GameTextures<'a> {
    pub fn load(texture_creator: &'a TextureCreator<WindowContext>) -> Result<Self, String> {
        let background = texture_creator.load_texture("assets/map.png")?;
        let car = texture_creator.load_texture("assets/car.png")?;

        Ok(GameTextures {
            background,
            car,
        })
    }
}

fn main() -> Result<(), String> {
    // Initialize SDL2
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let _image_context = sdl2::image::init(sdl2::image::InitFlag::PNG | sdl2::image::InitFlag::JPG)?;

    // Create window
    let window = video_subsystem
        .window("Smart Road", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;

    // Create texture creator
    let texture_creator = canvas.texture_creator();
    let textures = GameTextures::load(&texture_creator)?;

    let mut event_pump = sdl_context.event_pump()?;
    let mut game = Game::new();
    
    let mut last_time = Instant::now();

    // Main game loop
    'running: loop {
        let current_time = Instant::now();
        let delta_time = current_time.duration_since(last_time).as_secs_f32();
        last_time = current_time;

        // Handle events
        if !game.handle_events(&mut event_pump) {
            break 'running;
        }

        // Update game state
        game.update(delta_time);

        // Render
        game.render(&mut canvas, &textures)?;

        // Cap frame rate to ~60 FPS
        std::thread::sleep(Duration::from_millis(16));
    }

    Ok(())
}