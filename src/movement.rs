use crate::types::*;
use std::collections::HashMap;
use std::f32::consts::PI;

pub fn move_straight(car: &mut Car, delta_time: f32, in_intersection: &mut HashMap<CollisionType, Vec<usize>>) {
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

pub fn move_right(car: &mut Car, delta_time: f32, _in_intersection: &mut HashMap<CollisionType, Vec<usize>>) {
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

pub fn move_left(car: &mut Car, delta_time: f32, in_intersection: &mut HashMap<CollisionType, Vec<usize>>) {
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
                        car.rotation = PI / 2.0;
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
                        car.rotation = -PI / 2.0;
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