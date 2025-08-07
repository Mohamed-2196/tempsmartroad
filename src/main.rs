use sdl2::event::Event;
use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};
use std::collections::HashMap;
use std::time::{Duration, Instant};

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
}

impl Game {
    pub fn new() -> Self {
        Game {
            app_state: AppState::Running,
            stats: Stats::default(),
            start_time: Instant::now(),
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
                _ => {}
            }
        }
        
        // Check if we should exit
        self.app_state != AppState::Exit
    }

    pub fn update(&mut self, _delta_time: f32) {
        match self.app_state {
            AppState::Running => {
                // Game update logic will go here
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

        // Cars will be rendered here later

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