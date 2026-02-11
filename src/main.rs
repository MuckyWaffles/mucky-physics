use rand::prelude::*;
use raylib::ffi::CheckCollisionCircleLine;
use raylib::prelude::*;
mod physics;
use physics::*;
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

const SCREEN_WIDTH: i32 = 1400;
const SCREEN_HEIGHT: i32 = 800;

const PARTICLE_LIMIT: usize = 10000;

// const GRAVITY: f32 = 10.0;
const GRAVITY: f32 = 1.0;

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

    while !rl.window_should_close() {
        let (snap, data) = rx.recv().unwrap();

        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::BLACK);

        let plane_pos = snap.plane;
        // I really wish this function could operate in radians
        d.draw_circle_sector(plane_pos, 240.0, 300.0, 240.0, 20, Color::WHITE);
        d.draw_circle_sector(plane_pos, 210.0, 300.0, 240.0, 20, Color::BLACK);

        d.draw_circle_lines_v(plane_pos, 240.0, Color::PURPLE);
        d.draw_circle_lines_v(plane_pos, 210.0, Color::PURPLE);
        let tri_x = 240.0 * f32::sin(PI as f32 * 0.194);
        let tri_start = Vector2 {
            x: -tri_x + plane_pos.x,
            y: -240.0 + plane_pos.y,
        };
        let tri_end = Vector2 {
            x: tri_x + plane_pos.x,
            y: -240.0 + plane_pos.y,
        };
        d.draw_line_v(plane_pos, tri_start, Color::PURPLE);
        d.draw_line_v(plane_pos, tri_end, Color::PURPLE);
        d.draw_line_v(tri_start, tri_end, Color::PURPLE);

        // Render every particle
        for particle in snap.particles.iter() {
            render_particle(&mut d, particle);

            let part_pos = particle.pos - plane_pos;
            let angle = f32::atan2(part_pos.x, part_pos.y);

            let start = f32::atan2(tri_start.x - plane_pos.x, tri_start.y - plane_pos.y);
            let end = f32::atan2(tri_end.x - plane_pos.x, tri_end.y - plane_pos.y);

            if angle > start && angle < end {
            } else {
                d.draw_line_v(particle.pos, plane_pos, Color::PURPLE);
            }
        }
        for wall in snap.walls.iter() {
            d.draw_rectangle_rec(wall, Color::WHITE);
        }

        let str = format!(
            "Total Vector Velocity: ({}, {})",
            data.total_vector_vel.x, data.total_vector_vel.y,
        );
        let str2 = format!("Total Scalar Velocity: ({})", data.total_scalar_vel);
        let str3 = format!("Computation time (ms): {}", data.elapsed_time);
        let str4 = format!("Particle collision time (ms): {}", data.collision_time);

        d.draw_text(&str, 10, 40, 18, Color::LIGHTGRAY);
        d.draw_text(&str2, 10, 80, 18, Color::LIGHTGRAY);
        d.draw_text(&str3, 10, 120, 18, Color::LIGHTGRAY);
        d.draw_text(&str4, 10, 160, 18, Color::LIGHTGRAY);
    }
}

struct RenderParticle {
    pos: Vector2,
    radius: f32,
}

fn physics_thread(tx: mpsc::Sender<(Snapshot, Data)>) {
    let dt = 0.02;

    let mut particles = Vec::with_capacity(PARTICLE_LIMIT);

    let mut rng = rand::rng();
    let mut particles_alive: usize = 0;

    let walls: Vec<Rectangle> = Vec::with_capacity(0);

    // let ship_mass = particles_alive as f32 * 0.8;
    let ship_mass = 40.0;
    let mut ship_vel = Vector2 { x: 0.0, y: 0.0 };
    let mut plane_pos = Vector2 {
        x: SCREEN_WIDTH as f32 * 0.5,
        y: SCREEN_HEIGHT as f32 * 0.5,
    };

    let mut cells: Vec<Cell> = Vec::with_capacity(32);
    let width = SCREEN_WIDTH as f32 / 8.0;
    let height = SCREEN_HEIGHT as f32 / 8.0;

    for i in 0..128 {
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

    let mut elapsed: u64 = 0;
    loop {
        let now = Instant::now();

        let mut data = Data {
            total_vector_vel: Vector2 { x: 0.0, y: 0.0 },
            total_scalar_vel: 0.0,
            elapsed_time: elapsed,
            collision_time: 0,
        };
        let mut ship_impulse = Vector2 { x: -0.1, y: 0.0 };

        // Create new particles
        particles.push(Particle::new(Vector2 { x: 0.0, y: 0.0 }));
        particles[particles_alive] = Particle::new(Vector2 {
            x: -10.0,
            y: rng.random_range(10.0..SCREEN_HEIGHT as f32 - 10.0),
        });
        // Give new particles a starting velocity in a random direction
        let rand_dir = Vector2 {
            x: rng.random_range(-1.0..1.0),
            y: rng.random_range(-1.0..1.0),
        }
        .normalized();

        particles[particles_alive].vel = Vector2 {
            // x: rand_dir.x * 200.0,
            // y: rand_dir.y * 170.0,
            x: 100.0,
            y: rand_dir.y * 10.0,
        };
        particles[particles_alive].mass = 0.1;

        particles_alive += 1;

        // Update every particle
        for part in particles[0..particles_alive].iter_mut() {
            //part.force.y += GRAVITY * part.mass;
            part.integrate(dt);
        }

        let mut remove_queue: Vec<usize> = Vec::with_capacity(particles_alive);

        // Boundaries
        for (i, part) in particles[0..particles_alive].iter_mut().enumerate() {
            if part.pos.x + part.radius > SCREEN_WIDTH as f32 * 2.0 {
                remove_queue.push(i);
            } else if part.pos.x - part.radius < -SCREEN_WIDTH as f32 {
                remove_queue.push(i);
            }
            if part.pos.y + part.radius > SCREEN_HEIGHT as f32 {
                part.pos.y = SCREEN_HEIGHT as f32 - part.radius * 2.0;
                part.vel.y = -part.vel.y;
            } else if part.pos.y - part.radius < -SCREEN_HEIGHT as f32 {
                remove_queue.push(i);
            }
        }
        for i in remove_queue {
            particles.swap_remove(i);
            particles_alive -= 1;
        }

        for part in particles[0..particles_alive].iter_mut() {
            // particle_collide_wall(part, &mut walls, ship_mass, ship_vel, &mut ship_impulse);

            let zero = Vector2 { x: 0.0, y: 0.0 };
            let norm = particle_plane_normal(part, plane_pos);
            if norm == zero {
                continue;
            }

            // let mut new_ship_vel = Vector2 { x: 0.0, y: 0.0 };
            physics::collide_with_mass(norm, &mut part.vel, part.mass, &mut ship_vel, ship_mass);
            // ship_impulse += new_ship_vel - ship_vel;
        }

        //ship_impulse.y += GRAVITY;
        ship_vel += ship_impulse;
        // for wall in walls.iter_mut() {
        // wall.x += ship_vel.x * dt;
        // wall.y += ship_vel.y * dt;
        // }
        plane_pos += ship_vel * dt;

        if plane_pos.y - 190.0 > SCREEN_HEIGHT as f32 {
            plane_pos.y = SCREEN_HEIGHT as f32 + 190.0;
        }

        // Building cells
        for cell in cells.iter_mut() {
            cell.particles.clear();
        }
        for i in 0..particles_alive {
            let p = &particles[i];

            for cell in cells.iter_mut() {
                if cell.rect.check_collision_circle_rec(p.pos, p.radius) {
                    cell.particles.push(i);
                }
            }
        }

        for part in particles[0..particles_alive].iter_mut() {
            data.total_vector_vel += part.vel;
            data.total_scalar_vel += part.vel.length();
        }

        // Handling all collisions between particles
        let collide_start = Instant::now();
        for cell in cells.iter() {
            let indices = &cell.particles;

            for a_idx in 0..indices.len() {
                for b_idx in (a_idx + 1)..indices.len() {
                    let i = indices[a_idx];
                    let j = indices[b_idx];

                    let (left, right) = particles.split_at_mut(j);
                    let (pa, pb) = { (&mut left[i], &mut right[0]) };

                    Particle::collide(pa, pb);
                }
            }
        }
        data.collision_time = collide_start.elapsed().as_millis() as u64;

        let particle_snap: Vec<RenderParticle> = particles[0..particles_alive]
            .iter()
            .map(|p| RenderParticle {
                pos: p.pos,
                radius: p.radius,
            })
            .collect();
        let wall_snap: Vec<Rectangle> = walls.clone();

        let snapshot = Snapshot {
            particles: particle_snap,
            walls: wall_snap,
            plane: plane_pos,
        };
        tx.send((snapshot, data)).unwrap();

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
    particles: Vec<RenderParticle>,
    walls: Vec<Rectangle>,
    plane: Vector2,
}

struct Data {
    total_vector_vel: Vector2,
    total_scalar_vel: f32,
    elapsed_time: u64,
    collision_time: u64,
}

fn render_particle(d: &mut RaylibDrawHandle, particle: &RenderParticle) -> () {
    d.draw_circle_v(particle.pos, particle.radius, Color::WHITE)
}

fn particle_check_wall(part: &Particle, wall: &Rectangle) -> Vector2 {
    let left = Vector2 { x: -1.0, y: 0.0 };
    let right = Vector2 { x: 1.0, y: 0.0 };
    let up = Vector2 { x: 0.0, y: -1.0 };
    let down = Vector2 { x: 0.0, y: 1.0 };

    if !wall.check_collision_circle_rec(part.pos, part.radius) {
        return Vector2 { x: 0.0, y: 0.0 };
    }

    let l = wall.x + wall.width - (part.pos.x - part.radius);
    let r = part.pos.x + part.radius - wall.x;
    let u = wall.y + wall.height - (part.pos.y - part.radius);
    let d = part.pos.y + part.radius - wall.y;

    let min = f32::min(f32::min(l, r), f32::min(u, d));

    if min == l {
        return right;
    } else if min == r {
        return left;
    } else if min == u {
        return down;
    } else {
        return up;
    }
}

fn create_ship(ship: Rectangle) -> Vec<Rectangle> {
    let mut walls = Vec::with_capacity(5);
    walls.push(Rectangle {
        x: ship.x - 20.0,
        y: ship.y,
        width: 20.0,
        height: ship.height,
    });
    walls.push(Rectangle {
        x: ship.x,
        y: ship.y - 20.0,
        width: ship.width,
        height: 20.0,
    });
    walls.push(Rectangle {
        x: ship.x,
        y: ship.y + ship.height,
        width: ship.width,
        height: 20.0,
    });
    walls.push(Rectangle {
        x: ship.x + ship.width,
        y: ship.y,
        width: 20.0,
        height: 75.0,
    });
    walls.push(Rectangle {
        x: ship.x + ship.width,
        y: ship.y + ship.height - 75.0,
        width: 20.0,
        height: 75.0,
    });

    walls
}

fn particle_plane_normal(part: &Particle, plane_pos: Vector2) -> Vector2 {
    let delta = plane_pos - part.pos;
    let dist = delta.length();
    let plane_radius = 240.0;

    if (dist > plane_radius + part.radius) {
        return Vector2 { x: 0.0, y: 0.0 };
    }

    let mut norm = delta / dist;

    let tri_x = 240.0 * f32::sin(PI as f32 * 0.194);
    let tri_y = 240.0 * f32::cos(PI as f32 * 0.194);
    let tri_start = Vector2 {
        x: -tri_x + plane_pos.x,
        y: tri_y + plane_pos.y,
    };
    let tri_end = Vector2 {
        x: tri_x + plane_pos.x,
        y: tri_y + plane_pos.y,
    };

    let part_pos = part.pos - plane_pos;
    let angle = f32::atan2(part_pos.x, part_pos.y);

    let start = f32::atan2(-tri_x, tri_y) - PI as f32;
    let end = f32::atan2(tri_x, tri_y) - PI as f32;

    if angle > start && angle < end {
        if f32::abs(angle - start) < 5.0 {
            norm = Vector2 { x: -0.6, y: 0.3 };
        } else if f32::abs(angle - end) < 5.0 {
            norm = Vector2 { x: 0.6, y: 0.3 };
        }
    } else {
        norm = Vector2 { x: 0.0, y: 0.0 };
    }

    return norm;
}

fn particle_collide_wall(
    part: &mut Particle,
    walls: &mut Vec<Rectangle>,
    ship_mass: f32,
    ship_vel: Vector2,
    ship_impulse: &mut Vector2,
) {
    let left = Vector2 { x: -1.0, y: 0.0 };
    let right = Vector2 { x: 1.0, y: 0.0 };
    let up = Vector2 { x: 0.0, y: -1.0 };
    let down = Vector2 { x: 0.0, y: 1.0 };

    let zero = Vector2 { x: 0.0, y: 0.0 };
    for wall in walls.iter() {
        let m1 = ship_mass;
        let m2 = part.mass;

        let norm = particle_check_wall(part, wall);
        if norm == zero {
            continue;
        }

        let v1 = ship_vel.x;
        let v2 = part.vel.x;
        if norm == right || norm == left {
            let vcm = (v1 * m1 + v2 * m2) / (m1 + m2);
            ship_impulse.x += (2.0 * vcm - v1) - ship_vel.x;
            part.vel.x = 2.0 * vcm - v2;

            if norm == right {
                part.pos.x = wall.x + wall.width + part.radius;
            } else if norm == left {
                part.pos.x = wall.x - part.radius;
            }
        }

        let v1 = ship_vel.y;
        let v2 = part.vel.y;
        if norm == down || norm == up {
            let vcm = (v1 * m1 + v2 * m2) / (m1 + m2);
            ship_impulse.y += (2.0 * vcm - v1) - ship_vel.y;
            part.vel.y = 2.0 * vcm - v2;

            if norm == down {
                part.pos.y = wall.y + wall.height + part.radius;
            } else if norm == up {
                part.pos.y = wall.y - part.radius;
            }
        }
    }
}
