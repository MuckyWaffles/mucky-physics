use rand::prelude::*;
use raylib::prelude::*;
mod physics;
use physics::*;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

const SCREEN_WIDTH: i32 = 1400;
const SCREEN_HEIGHT: i32 = 800;

const PARTICLE_LIMIT: usize = 10000;

// const GRAVITY: f32 = 10.0;
const GRAVITY: f32 = 0.0;

fn main() {
    // You know what I really like Rust syntax
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Mucky Physics")
        .msaa_4x()
        .build();

    rl.set_target_fps(75);

    // create send/receiver cars to move data
    let (tx, rx) = mpsc::channel();
    thread::spawn(|| physics_thread(tx));

    let mut show_info = true;

    while !rl.window_should_close() {
        let (snap, data) = rx.recv().unwrap();

        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::BLACK);

        for body in snap.bodies.iter() {
            match body.shape {
                Shape::Circle(a) => d.draw_circle_v(body.motion.pos, a.radius, Color::WHITE),
                Shape::CircleSeg(a) => {
                    d.draw_circle_v(body.motion.pos, a.radius, Color::WHITE);
                    d.draw_rectangle(
                        (body.motion.pos.x - a.radius) as i32,
                        (body.motion.pos.y) as i32,
                        (a.radius * 2.0) as i32,
                        (a.radius * 2.0) as i32,
                        Color::BLACK,
                    );
                }
            }
        }

        let options_window = Rectangle {
            x: SCREEN_WIDTH as f32 - 180.0,
            y: SCREEN_HEIGHT as f32 - 180.0,
            width: 160.0,
            height: 160.0,
        };
        _ = d.gui_window_box(options_window, "Menu");

        if d.gui_label_button(
            Rectangle {
                x: options_window.x + 20.0,
                y: options_window.y + 30.0,
                width: 80.0,
                height: 20.0,
            },
            "Show Info",
        ) {
            show_info = !show_info;
        }

        if show_info {
            if d.gui_window_box(
                Rectangle {
                    x: 10.0,
                    y: 10.0,
                    width: 400.0,
                    height: 240.0,
                },
                "Menu",
            ) {
                show_info = false;
            }

            let str = format!(
                "Total Vector Velocity: ({}, {})",
                data.total_vector_vel.x, data.total_vector_vel.y,
            );
            let str2 = format!("Total Scalar Velocity: ({})", data.total_scalar_vel);
            let str3 = format!("Computation time (ms): {}", data.elapsed_time);
            let str4 = format!("Particle collision time (ms): {}", data.collision_time);
            // let str5 = format!("plane height: {}", plane_pos.y);

            let text_color = Color::DARKSLATEGRAY;
            d.draw_text(&str, 20, 50, 18, text_color);
            d.draw_text(&str2, 20, 90, 18, text_color);
            d.draw_text(&str3, 20, 130, 18, text_color);
            d.draw_text(&str4, 20, 170, 18, text_color);
            // d.draw_text(&str5, 20, 210, 18, text_color);
        }
    }
}

fn physics_thread(tx: mpsc::Sender<(Snapshot, Data)>) {
    let dt = 0.02;

    let mut bodies: Vec<Body> = Vec::with_capacity(PARTICLE_LIMIT);
    let mut bodies_alive: usize = 0;

    let mut cells: Vec<Cell> = Vec::with_capacity(32);
    let width = SCREEN_WIDTH as f32 / 8.0;
    let height = SCREEN_HEIGHT as f32 / 8.0;

    for i in 0..64 {
        cells.push(Cell {
            particles: Vec::with_capacity(PARTICLE_LIMIT),
            rect: Rectangle {
                x: (i % 8) as f32 * width,
                y: (i / 8) as f32 * height,
                width,
                height,
            },
        });
    }

    let radius = 180.0;
    bodies.push(Body::new(
        Vector2 {
            x: SCREEN_WIDTH as f32 / 2.0,
            y: SCREEN_HEIGHT as f32 / 2.0,
        },
        Shape::CircleSeg(CircleSeg::new(radius, Vector2 { x: 0.242, y: 0.97 })),
    ));
    bodies[0].motion.mass = 100.0;

    let mut elapsed: u64 = 0;
    loop {
        let now = Instant::now();

        let mut data = Data {
            total_vector_vel: Vector2 { x: 0.0, y: 0.0 },
            total_scalar_vel: 0.0,
            elapsed_time: elapsed,
            collision_time: 0,
        };

        // Update every body
        for body in bodies[0..bodies_alive].iter_mut() {
            body.motion.force.y += GRAVITY * body.motion.mass;
            body.motion.integrate(dt);
        }

        // Create new particles
        let mut new_bodies = body_emitter(bodies.clone(), &mut bodies_alive);
        let mut remove_queue: Vec<usize> = Vec::with_capacity(bodies_alive);

        for (i, body) in new_bodies[0..bodies_alive].iter_mut().enumerate() {
            if body.motion.pos.x > SCREEN_WIDTH as f32 * 2.0 {
                remove_queue.push(i);
            }
        }

        for i in remove_queue.iter().rev() {
            new_bodies.swap_remove(*i);
            bodies_alive -= 1;
        }

        // Building cells
        for cell in cells.iter_mut() {
            cell.particles.clear();
        }
        for i in 0..bodies_alive {
            let b = &new_bodies[i];

            for cell in cells.iter_mut() {
                match b.shape {
                    Shape::Circle(a) => {
                        if cell.rect.check_collision_circle_rec(b.motion.pos, a.radius) {
                            cell.particles.push(i);
                        }
                    }
                    Shape::CircleSeg(a) => {
                        if cell.rect.check_collision_circle_rec(b.motion.pos, a.radius) {
                            cell.particles.push(i);
                        }
                    }
                }
            }
        }

        for body in new_bodies[0..bodies_alive].iter_mut() {
            data.total_vector_vel += body.motion.vel;
            data.total_scalar_vel += body.motion.vel.length();
        }

        // Handling all collisions between particles
        let collide_start = Instant::now();
        for cell in cells.iter() {
            let indices = &cell.particles;

            for a_idx in 0..indices.len() {
                for b_idx in (a_idx + 1)..indices.len() {
                    let i = indices[a_idx];
                    let j = indices[b_idx];

                    let (left, right) = new_bodies.split_at_mut(j);
                    let (pa, pb) = { (&mut left[i], &mut right[0]) };

                    let norm = Shape::collide(
                        &pa.shape,
                        pa.motion.pos,
                        pa.motion.vel,
                        &pb.shape,
                        pb.motion.pos,
                        pb.motion.vel,
                    )
                    .unwrap_or(Vector2 { x: 0.0, y: 0.0 });

                    collide_with_mass(
                        norm,
                        &mut pa.motion.vel,
                        pa.motion.mass,
                        &mut pb.motion.vel,
                        pb.motion.mass,
                    );
                }
            }
        }

        data.collision_time = collide_start.elapsed().as_millis() as u64;

        let snapshot = Snapshot {
            bodies: new_bodies.clone(),
        };
        tx.send((snapshot, data)).unwrap();

        bodies = new_bodies;

        elapsed = now.elapsed().as_millis() as u64;
        let sleep_time = i64::max(20 - elapsed as i64, 0);
        thread::sleep(Duration::from_millis(sleep_time as u64));
    }
}

struct Cell {
    particles: Vec<usize>,
    rect: Rectangle,
}

struct Snapshot {
    bodies: Vec<Body>,
}

struct Data {
    total_vector_vel: Vector2,
    total_scalar_vel: f32,
    elapsed_time: u64,
    collision_time: u64,
}

// fn particle_check_wall(part: &Particle, wall: &Rectangle) -> Vector2 {
//     let left = Vector2 { x: -1.0, y: 0.0 };
//     let right = Vector2 { x: 1.0, y: 0.0 };
//     let up = Vector2 { x: 0.0, y: -1.0 };
//     let down = Vector2 { x: 0.0, y: 1.0 };

//     if !wall.check_collision_circle_rec(part.pos, part.radius) {
//         return Vector2 { x: 0.0, y: 0.0 };
//     }

//     let l = wall.x + wall.width - (part.pos.x - part.radius);
//     let r = part.pos.x + part.radius - wall.x;
//     let u = wall.y + wall.height - (part.pos.y - part.radius);
//     let d = part.pos.y + part.radius - wall.y;

//     let min = f32::min(f32::min(l, r), f32::min(u, d));

//     if min == l {
//         return right;
//     } else if min == r {
//         return left;
//     } else if min == u {
//         return down;
//     } else {
//         return up;
//     }
// }

// fn particle_collide_wall(
//     part: &mut Particle,
//     walls: &mut Vec<Rectangle>,
//     ship_mass: f32,
//     ship_vel: Vector2,
//     ship_impulse: &mut Vector2,
// ) {
//     let left = Vector2 { x: -1.0, y: 0.0 };
//     let right = Vector2 { x: 1.0, y: 0.0 };
//     let up = Vector2 { x: 0.0, y: -1.0 };
//     let down = Vector2 { x: 0.0, y: 1.0 };

//     let zero = Vector2 { x: 0.0, y: 0.0 };
//     for wall in walls.iter() {
//         let m1 = ship_mass;
//         let m2 = part.mass;

//         let norm = particle_check_wall(part, wall);
//         if norm == zero {
//             continue;
//         }

//         let v1 = ship_vel.x;
//         let v2 = part.vel.x;
//         if norm == right || norm == left {
//             let vcm = (v1 * m1 + v2 * m2) / (m1 + m2);
//             ship_impulse.x += (2.0 * vcm - v1) - ship_vel.x;
//             part.vel.x = 2.0 * vcm - v2;

//             if norm == right {
//                 part.pos.x = wall.x + wall.width + part.radius;
//             } else if norm == left {
//                 part.pos.x = wall.x - part.radius;
//             }
//         }

//         let v1 = ship_vel.y;
//         let v2 = part.vel.y;
//         if norm == down || norm == up {
//             let vcm = (v1 * m1 + v2 * m2) / (m1 + m2);
//             ship_impulse.y += (2.0 * vcm - v1) - ship_vel.y;
//             part.vel.y = 2.0 * vcm - v2;

//             if norm == down {
//                 part.pos.y = wall.y + wall.height + part.radius;
//             } else if norm == up {
//                 part.pos.y = wall.y - part.radius;
//             }
//         }
//     }
// }

const PARTICLES_PER_TICK: u32 = 2;

fn body_emitter(mut bodies: Vec<Body>, bodies_alive: &mut usize) -> Vec<Body> {
    let mut rng = rand::rng();

    let radius = 8.0;

    for _ in 0..PARTICLES_PER_TICK {
        let new = bodies.len();
        bodies.push(Body::new(
            Vector2 {
                x: -radius,
                y: rng.random_range(radius..SCREEN_HEIGHT as f32 - 10.0),
            },
            Shape::Circle(Circle::new(radius)),
        ));

        let rand_dir = Vector2 {
            x: rng.random_range(-1.0..1.0),
            y: rng.random_range(-1.0..1.0),
        }
        .normalized();

        bodies[new].motion.vel = Vector2 {
            x: 100.0 + rand_dir.x * 10.0,
            y: rand_dir.y * 10.0,
        };
        bodies[new].motion.mass = 0.1;

        *bodies_alive += 1;
    }

    return bodies;
}
