use std::time::{Duration, Instant};

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