use sdl2::event::Event;
use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::{Canvas, Texture, TextureCreator};
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
}

impl Game {
    pub fn new() -> Self {
        let mut spawn_coords = HashMap::new();
        
        // Spawn coordinates from Bevy version
        spawn_coords.insert((Route::Right, Direction::North), (655.0, 0.0));
        spawn_coords.insert((Route::Right, Direction::West), (1024.0, 452.0));
        spawn_coords.insert((Route::Right, Direction::East), (0.0, 250.0));
        spawn_coords.insert((Route::Right, Direction::South), (360.0, 682.0));
        spawn_coords.insert((Route::Straight, Direction::North), (595.0, 45.0));
        spawn_coords.insert((Route::Straight, Direction::West), (978.0, 407.0));
        spawn_coords.insert((Route::Straight, Direction::East), (0.0, 292.0));
        spawn_coords.insert((Route::Straight, Direction::South), (420.0, 682.0));
        spawn_coords.insert((Route::Left, Direction::North), (535.0, 45.0));
        spawn_coords.insert((Route::Left, Direction::West), (978.0, 364.0));
        spawn_coords.insert((Route::Left, Direction::East), (0.0, 332.0));
        spawn_coords.insert((Route::Left, Direction::South), (480.0, 682.0));
        
        Game {
            app_state: AppState::Running,
            stats: Stats::default(),
            start_time: Instant::now(),
            cars: Vec::new(),
            next_car_id: 1,
            spawn_coords,
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
                // Update cars with route-specific movement
                for car in &mut self.cars {
                    if car.moving {
                        match car.route {
                            Route::Straight => Self::move_straight(car, delta_time),
                            Route::Right => Self::move_right(car, delta_time),
                            Route::Left => Self::move_left(car, delta_time),
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
                        cars_to_remove.push(i);
                    }
                }
                
                // Remove cars in reverse order to maintain indices
                for &i in cars_to_remove.iter().rev() {
                    self.cars.remove(i);
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
        // Clear with black background
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        // Draw intersection background
        let dst_rect = sdl2::rect::Rect::new(0, 0, WINDOW_WIDTH, WINDOW_HEIGHT);
        canvas.copy(&textures.background, None, Some(dst_rect))?;

        // Draw cars with rotation
        for car in &self.cars {
            let car_width = 46;
            let car_height = 90;
            let dst_rect = sdl2::rect::Rect::new(
                (car.x - car_width as f32 / 2.0) as i32,
                (car.y - car_height as f32 / 2.0) as i32,
                car_width,
                car_height,
            );
            
            // Convert rotation from radians to degrees for SDL2
            let angle_degrees = car.rotation * 180.0 / PI;
            
            // Draw with rotation
            canvas.copy_ex(
                &textures.car,
                None,
                Some(dst_rect),
                angle_degrees as f64,
                None,
                false,
                false,
            )?;
        }

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
                };
                
                self.cars.push(car);
                self.next_car_id += 1;
                self.stats.max_number_cars += 1;
                
                println!("Spawned car {} at ({}, {})", self.next_car_id - 1, x, y);
            }
        }
    }
    
    // Movement functions based on Bevy version
    fn move_straight(car: &mut Car, delta_time: f32) {
        match car.direction {
            Direction::North => {
                if car.y < 682.0 {
                    car.y += car.speed * delta_time;
                } else {
                    car.moving = false;
                }
            }
            Direction::South => {
                if car.y > 0.0 {
                    car.y -= car.speed * delta_time;
                } else {
                    car.moving = false;
                }
            }
            Direction::East => {
                if car.x < 1023.0 {
                    car.x += car.speed * delta_time;
                } else {
                    car.moving = false;
                }
            }
            Direction::West => {
                if car.x > 0.0 {
                    car.x -= car.speed * delta_time;
                } else {
                    car.moving = false;
                }
            }
        }
    }
    
    fn move_right(car: &mut Car, delta_time: f32) {
        match car.direction {
            Direction::North => {
                if car.y < 247.0 {
                    car.y += car.speed * delta_time;
                    car.speed -= 0.2;
                    if car.y > 170.0 {
                        car.x += 0.3;
                        if car.rotation > -PI / 2.0 {
                            car.rotation -= 2.7 * delta_time;
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
                        if car.rotation.abs() > 0.02 {
                            car.rotation -= 2.78 * delta_time;
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
                if car.y > 450.0 {
                    car.y -= car.speed * delta_time;
                    car.speed -= 0.2;
                    if car.y < 540.0 {
                        car.x -= 0.2;
                        if car.rotation.abs() > PI / 2.0 {
                            car.rotation -= 2.3 * delta_time;
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
                        if car.rotation.abs() < 3.1 {
                            car.rotation -= 2.85 * delta_time;
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
    
    fn move_left(car: &mut Car, delta_time: f32) {
        match car.direction {
            Direction::North => {
                if car.y < 365.0 {
                    car.y += car.speed * delta_time;
                    if car.y > 307.0 {
                        if car.rotation.abs() < 3.0 * PI / 4.0 {
                            car.rotation += 4.0 * delta_time;
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
                if car.y > 330.0 {
                    car.y -= car.speed * delta_time;
                    if car.y < 388.0 {
                        if car.rotation.abs() > PI / 2.0 {
                            car.rotation += 4.0 * delta_time;
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
                    car.x += car.speed * delta_time;
                    if car.x > 490.0 {
                        if car.rotation.abs() < PI {
                            car.rotation += 4.7 * delta_time;
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
                    car.x -= car.speed * delta_time;
                    if car.x < 535.0 {
                        if car.rotation.abs() < PI {
                            car.rotation += 4.0 * delta_time;
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