use raylib::prelude::*;

const SCREEN_WIDTH: i32 = 1000;
const SCREEN_HEIGHT: i32 = 600;

struct Particle {
    in_use: bool,
    
    previous: Vector2,
    position: Vector2,
    accel: Vector2,
    force: Vector2,
    mass: f32,
    radius: f32,
}

const PARTICLE_LIMIT: usize = 1000;
fn new_particle(
    particles: &mut [Particle; PARTICLE_LIMIT],
    position: Vector2,
) -> Option<&mut Particle> {
    for particle in particles.iter_mut() {
        if particle.in_use {
            continue;
        }

        *particle = Particle {
            in_use: true,
            previous: position,
            position,
            accel: Vector2 { x: 0.0, y: 0.0 },
            force: Vector2 { x: 0.0, y: 0.0 },
            mass: 1.0,
            radius: 10.0,
        };

        return Some(particle);
    }

    None // no free particle available
}

fn main() {
	let mut particles: [Particle; PARTICLE_LIMIT];

    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Mucky Physics")
        .build();

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::BLACK);
    }
}
