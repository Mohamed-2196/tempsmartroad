use crate::types::*;
use sdl2::image::LoadTexture;
use sdl2::pixels::Color;
use sdl2::render::{Canvas, Texture, TextureCreator, BlendMode};
use sdl2::video::{Window, WindowContext};
use sdl2::ttf::Font;
use std::f32::consts::PI;

pub const WINDOW_WIDTH: u32 = 1024;
pub const WINDOW_HEIGHT: u32 = 682;

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

pub fn render_game(canvas: &mut Canvas<Window>, textures: &GameTextures, cars: &[Car]) -> Result<(), String> {
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
        for car in cars {
            let car_width = 32;
            let car_height = 60;
            let dst_rect = sdl2::rect::Rect::new(
                (car.x - car_width as f32 / 2.0) as i32,
                (car.y - car_height as f32 / 2.0) as i32,
                car_width,
                car_height,
            );
            
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
    
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    
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

pub fn render_stats(canvas: &mut Canvas<Window>, font: &Font, stats: &Stats) -> Result<(), String> {
    // Clear with black background
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    let texture_creator = canvas.texture_creator();
    
    // Title
    let title_surface = font.render("Simulation Statistics")
        .blended(Color::WHITE)
        .map_err(|e| e.to_string())?;
    let title_texture = texture_creator.create_texture_from_surface(&title_surface)
        .map_err(|e| e.to_string())?;
    let title_rect = sdl2::rect::Rect::new(300, 50, title_surface.width(), title_surface.height());
    canvas.copy(&title_texture, None, Some(title_rect))?;

    // Stats text
    let min_travel_time = if stats.min_time.as_secs_f32() == 1000.0 {
        0.0
    } else {
        stats.min_time.as_secs_f32()
    };
    
    let stats_lines = vec![
        format!("Total Cars: {}", stats.max_number_cars),
        format!("Maximum Speed: {:.2} units/s", stats.max_velocity),
        format!("Minimum Speed: {:.2} units/s", stats.min_velocity),
        format!("Maximum Travel Time: {:.2} seconds", stats.max_time.as_secs_f32()),
        format!("Minimum Travel Time: {:.2} seconds", min_travel_time),
        format!("Close Calls: {}", stats.close_call),
    ];

    // Render each stat line
    for (i, line) in stats_lines.iter().enumerate() {
        let surface = font.render(line)
            .blended(Color::WHITE)
            .map_err(|e| e.to_string())?;
        let texture = texture_creator.create_texture_from_surface(&surface)
            .map_err(|e| e.to_string())?;
        let rect = sdl2::rect::Rect::new(
            200, 
            150 + (i as i32) * 50, 
            surface.width(), 
            surface.height()
        );
        canvas.copy(&texture, None, Some(rect))?;
    }

    // Exit instruction
    let exit_surface = font.render("Press ESC again to exit")
        .blended(Color::WHITE)
        .map_err(|e| e.to_string())?;
    let exit_texture = texture_creator.create_texture_from_surface(&exit_surface)
        .map_err(|e| e.to_string())?;
    let exit_rect = sdl2::rect::Rect::new(
        350, 
        550, 
        exit_surface.width(), 
        exit_surface.height()
    );
    canvas.copy(&exit_texture, None, Some(exit_rect))?;

    canvas.present();
    Ok(())
}