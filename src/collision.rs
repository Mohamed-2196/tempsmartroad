use crate::types::*;
use std::collections::HashMap;

const HITBOX_BUFFER: f32 = 2.0;

pub fn build_car_tracking(car_data: &[(usize, f32, f32, Vec<CollisionType>, bool)]) -> HashMap<usize, Vec<(f32, f32, usize, CollisionType, bool)>> {
    let mut car_tracking: HashMap<usize, Vec<(f32, f32, usize, CollisionType, bool)>> = HashMap::new();
    
    for (id, _x, _y, types, _rotated) in car_data {
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

pub fn check_collision(
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
    let car_corners = compute_rotated_corners(car_x, car_y, primary_type, rotated);
    
    let mut temp_win = 0;
    
    if let Some(others) = car_tracking.get(&car_id) {
        for &(x, y, other_id, other_type, did_rotate) in others {
            let other_corners = compute_rotated_corners(x, y, other_type, did_rotate);
            
            if sat_collision(&car_corners, &other_corners) {
                let pair = (car_id.min(other_id), car_id.max(other_id));
                
                // Check existing winner from priority_map (This is where my algorithm takes place, given a set of reference points in a map,
                // I compare their distances to a reference point and record who wins (who is closer) between two cars and
                // make him the priority until the car leaves the intersection)
                if let Some(&winner) = priority_map.get(&pair) {
                    temp_win = winner;
                    if winner != car_id {
                        stats.close_call += 1;
                        return true;
                    }
                }
                
                // Determine reference point based on collision types
                let (ref_x, ref_y) = get_reference_point(primary_type, other_type);
                
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
                // Basically who respawned first
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
    update_reference_points(car_x, car_y, &car_corners, car_id, priority_ref);
    
    false
}

pub fn check_spawn_collision(
    x: f32,
    y: f32,
    collision_type: CollisionType,
    car_data: &[(usize, f32, f32, Vec<CollisionType>, bool)],
) -> bool {
    let car_corners = compute_rotated_corners(x, y, collision_type, false);
    
    for (_, ox, oy, other_types, did_rotate) in car_data {
        if let Some(&other_type) = other_types.get(0) {
            if collision_type == other_type {
                let other_corners = compute_rotated_corners(*ox, *oy, other_type, *did_rotate);
                if sat_collision(&car_corners, &other_corners) {
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
    _car_x: f32,
    _car_y: f32,
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
        if contains_point(car_corners, ref_x, ref_y) {
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
    
    let hw = width / 2.0 + HITBOX_BUFFER;
    let hh = height / 2.0 + HITBOX_BUFFER;
    
    [
        Vec2::new(x - hw, y - hh), // Bottom-left
        Vec2::new(x + hw, y - hh), // Bottom-right
        Vec2::new(x + hw, y + hh), // Top-right
        Vec2::new(x - hw, y + hh), // Top-left
    ]
}

//Could have used SDL2 Rect.contains() but I just love this ALGO 
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
        let (a_min, a_max) = project(a, axis);
        let (b_min, b_max) = project(b, axis);
        
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