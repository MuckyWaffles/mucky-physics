use rand::prelude::*;
use raylib::prelude::*;
mod physics;
use physics::Particle;

const SCREEN_WIDTH: i32 = 1000;
const SCREEN_HEIGHT: i32 = 600;

// TODO: Make this a packed pool
const PARTICLE_LIMIT: usize = 1000;
fn new_particle(particles: &mut Vec<Particle>, pos: Vector2) -> Option<&mut Particle> {
    for particle in particles.iter_mut().filter(|p| p.is_alive()) {
        *particle = Particle::new(pos);
        return Some(particle);
    }

    None // no free particle available
}

fn render_particle(d: &mut RaylibDrawHandle, particle: &mut Particle) -> () {
    d.draw_circle_v(particle.pos, particle.radius, Color::WHITE)
}

const GRAVITY: f32 = 10.0;

fn main() {
    // Creating empty particle array
    let mut particles = Vec::with_capacity(PARTICLE_LIMIT);
    for i in 0..PARTICLE_LIMIT {
        particles.push(Particle::new(Vector2 { x: 0.0, y: 0.0 }));
        particles[i].end();
    }

    // You know what I really like Rust syntax
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Mucky Physics")
        .msaa_4x()
        .build();

    rl.set_target_fps(60);
    let dt = 1.0 / 60.0;

    //let _ = new_particle(&mut particles, Vector2 { x: 100.0, y: 100.0 }).unwrap();

    let mut rng = rand::rng();
    let mut particles_alive: usize = 0;
    // This code is kinda sloppy but whatevs
    for _i in 0..100 {
        particles[particles_alive] = Particle::new(Vector2 {
            x: rng.random_range(10.0..1000.0 - 10.0),
            y: rng.random_range(10.0..600.0 - 10.0),
        });
        // Give new particles a starting velocity in a random direction
        let rand_dir = Vector2 {
            x: rng.random_range(-1.0..1.0),
            y: rng.random_range(-1.0..1.0),
        }
        .normalized();

        particles[particles_alive].vel = Vector2 {
            x: rand_dir.x * 40.0,
            y: rand_dir.y * 40.0,
        };
        particles_alive += 1;
    }

    while !rl.window_should_close() {
        // Update every particle
        for part in particles[0..particles_alive].iter_mut() {
            //part.force.y += GRAVITY * part.mass;
            part.integrate(dt);

            // Boundaries
            if part.pos.x + part.radius > SCREEN_WIDTH as f32 {
                part.pos.x = SCREEN_WIDTH as f32 - part.radius;
                part.vel.x = -part.vel.x;
            } else if part.pos.x - part.radius < 0.0 {
                part.pos.x = part.radius;
                part.vel.x = -part.vel.x;
            }
            if part.pos.y + part.radius > SCREEN_HEIGHT as f32 {
                part.pos.y = SCREEN_HEIGHT as f32 - part.radius;
                part.vel.y = -part.vel.y;
            } else if part.pos.y - part.radius < 0.0 {
                part.pos.y = part.radius;
                part.vel.y = -part.vel.y;
            }
        }

        // TODO: make a new collision detection system
        // with separate broad-phase and narrow-phase for optimization
        for i in 0..particles_alive {
            // Woah, Rust is weird sometimes...
            // We split here so we never iterate over the same two particles twice
            let (left, right) = particles[..particles_alive].split_at_mut(i + 1);
            let a = &mut left[i];

            right.iter_mut().for_each(|b| Particle::collide(a, b));
        }

        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::BLACK);

        // Render every particle
        for particle in particles.iter_mut().filter(|p| p.is_alive()) {
            render_particle(&mut d, particle);
        }
    }
}
