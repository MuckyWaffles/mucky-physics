use rand::prelude::*;
use raylib::prelude::*;
mod physics;
use physics::Particle;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

const SCREEN_WIDTH: i32 = 1400;
const SCREEN_HEIGHT: i32 = 800;

const PARTICLE_LIMIT: usize = 2000;

const GRAVITY: f32 = 10.0;

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

        // Render every particle
        for particle in snap.particles.iter() {
            render_particle(&mut d, particle);
        }
        for wall in snap.walls.iter() {
            d.draw_rectangle_rec(wall, Color::WHITE);
        }

        let str = format!(
            "Total Vector Velocity: ({}, {})",
            data.total_vector_vel.x, data.total_vector_vel.y,
        );
        let str2 = format!("Total Scalar Velocity: ({})", data.total_scalar_vel);

        d.draw_text(&str, 10, 40, 18, Color::LIGHTGRAY);
        d.draw_text(&str2, 10, 80, 18, Color::LIGHTGRAY);
    }
}

struct RenderParticle {
    pos: Vector2,
    radius: f32,
}

fn physics_thread(tx: mpsc::Sender<(Snapshot, Data)>) {
    let dt = 0.02;

    // Creating empty particle array
    let mut particles = Vec::with_capacity(PARTICLE_LIMIT);
    for _i in 0..PARTICLE_LIMIT {
        particles.push(Particle::new(Vector2 { x: 0.0, y: 0.0 }));
    }

    //let _ = new_particle(&mut particles, Vector2 { x: 100.0, y: 100.0 }).unwrap();

    let mut rng = rand::rng();
    let mut particles_alive: usize = 0;
    // This code is kinda sloppy but whatevs

    let screen_half = Vector2 {
        x: SCREEN_WIDTH as f32 / 2.0 + 400.0,
        y: SCREEN_HEIGHT as f32 / 2.0,
    };
    for _i in 0..1000 {
        particles[particles_alive] = Particle::new(Vector2 {
            x: rng.random_range(screen_half.x - 120.0..screen_half.x + 120.0),
            y: rng.random_range(screen_half.y - 100.0..screen_half.y + 100.0),
        });
        // Give new particles a starting velocity in a random direction
        let rand_dir = Vector2 {
            x: rng.random_range(-1.0..1.0),
            y: rng.random_range(-1.0..1.0),
        }
        .normalized();

        particles[particles_alive].vel = Vector2 {
            x: rand_dir.x * 200.0,
            y: rand_dir.y * 170.0,
        };
        particles_alive += 1;
    }

    let ship = Rectangle {
        x: screen_half.x - 120.0,
        y: screen_half.y - 90.0,
        width: 260.0,
        height: 180.0,
    };
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
    let left = Vector2 { x: -1.0, y: 0.0 };
    let right = Vector2 { x: 1.0, y: 0.0 };
    let up = Vector2 { x: 0.0, y: -1.0 };
    let down = Vector2 { x: 0.0, y: 1.0 };

    let ship_mass = particles_alive as f32 * 0.8;
    let mut ship_vel = Vector2 { x: 0.0, y: 0.0 };

    loop {
        let now = Instant::now();

        let mut data = Data {
            total_vector_vel: Vector2 { x: 0.0, y: 0.0 },
            total_scalar_vel: 0.0,
        };
        let mut ship_impulse = Vector2 { x: 0.0, y: 0.0 };

        // Update every particle
        for part in particles[0..particles_alive].iter_mut() {
            //part.force.y += GRAVITY * part.mass;
            part.integrate(dt);
        }

        // Boundaries
        for part in particles[0..particles_alive].iter_mut() {
            if part.pos.x + part.radius > SCREEN_WIDTH as f32 {
                part.pos.x = SCREEN_WIDTH as f32 - part.radius;
                part.vel.x = -part.vel.x;
            } else if part.pos.x - part.radius < 0.0 {
                // part.pos.x = part.radius;
                // part.vel.x = -part.vel.x;
            }
            if part.pos.y + part.radius > SCREEN_HEIGHT as f32 {
                part.pos.y = SCREEN_HEIGHT as f32 - part.radius;
                part.vel.y = -part.vel.y;
            } else if part.pos.y - part.radius < 0.0 {
                part.pos.y = part.radius;
                part.vel.y = -part.vel.y;
            }
        }

        let zero = Vector2 { x: 0.0, y: 0.0 };
        for part in particles[0..particles_alive].iter_mut() {
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

            data.total_vector_vel += part.vel;
            data.total_scalar_vel += part.vel.length();
        }

        ship_vel += ship_impulse;
        for wall in walls.iter_mut() {
            wall.x += ship_vel.x * dt;
            wall.y += ship_vel.y * dt;
        }

        // TODO: make a new collision detection system
        // with separate broad-phase and narrow-phase for optimization
        for i in 0..particles_alive {
            // We split here so we never iterate over the same two particles twice
            let (left, right) = particles[..particles_alive].split_at_mut(i + 1);
            let a = &mut left[i];

            right.iter_mut().for_each(|b| Particle::collide(a, b));
        }

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
        };
        tx.send((snapshot, data)).unwrap();

        let elapsed = now.elapsed().as_millis();
        println!("elapsed: {}", elapsed);
        let sleep_time = i64::max(20 - elapsed as i64, 0);
        thread::sleep(Duration::from_millis(sleep_time as u64));
    }
}

struct Snapshot {
    particles: Vec<RenderParticle>,
    walls: Vec<Rectangle>,
}

struct Data {
    total_vector_vel: Vector2,
    total_scalar_vel: f32,
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
