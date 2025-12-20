use raylib::prelude::*;
mod physics;
use physics::Particle;

const SCREEN_WIDTH: i32 = 1000;
const SCREEN_HEIGHT: i32 = 600;

// TODO: Make this a packed pool
const PARTICLE_LIMIT: usize = 1000;
fn new_particle(particles: &mut Vec<Particle>, pos: Vector2) -> Option<&mut Particle> {
    for particle in particles.iter_mut().filter(|p| p.is_active()) {
        *particle = Particle::new(pos);
        return Some(particle);
    }

    None // no free particle available
}

fn render_particle(d: &mut RaylibDrawHandle, particle: &mut Particle) -> () {
    d.draw_circle_v(particle.pos, particle.radius, Color::WHITE)
}

const GRAVITY: f32 = 0.1;

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

    let _ = new_particle(&mut particles, Vector2 { x: 100.0, y: 100.0 }).unwrap();

    while !rl.window_should_close() {
        // Update every particle
        for particle in particles.iter_mut().filter(|p| p.is_active()) {
            particle.force.y += GRAVITY * particle.mass;
            particle.integrate(dt);
        }

        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::BLACK);

        // Render every particle
        for particle in particles.iter_mut().filter(|p| p.is_active()) {
            render_particle(&mut d, particle);
        }
    }
}
