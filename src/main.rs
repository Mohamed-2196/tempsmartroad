mod types;
mod collision;
mod movement;
mod renderer;
mod game;

use game::Game;
use renderer::{GameTextures, WINDOW_WIDTH, WINDOW_HEIGHT};
use std::time::{Duration, Instant};

fn main() -> Result<(), String> {
    // Initialize SDL2
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let _image_context = sdl2::image::init(sdl2::image::InitFlag::PNG | sdl2::image::InitFlag::JPG)?;
    
    // Initialize TTF (font for the stats at the end)
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
    
    // Load font (using a system font - you can replace this path if you don't like it)
    let font = ttf_context.load_font("C:/Windows/Fonts/arial.ttf", 24)
        .or_else(|_| ttf_context.load_font("/System/Library/Fonts/Arial.ttf", 24))
        .or_else(|_| ttf_context.load_font("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 24))
        .map_err(|e| format!("Could not load font: {}", e))?;

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
        game.render(&mut canvas, &textures, &font)?;

        // Cap frame rate to ~60 FPS
        std::thread::sleep(Duration::from_millis(16));
    }

    Ok(())
}