use raylib::prelude::*;

const SCREEN_WIDTH: i32 = 1000;
const SCREEN_HEIGHT: i32 = 600;

struct Particle {
    active: bool,

    previous: Vector2,
    position: Vector2,
    accel: Vector2,
    force: Vector2,
    mass: f32,
    radius: f32,
}

impl Particle {
    fn new(position: Vector2) -> Particle {
        Particle {
            active: true,
            previous: position,
            position,
            accel: Vector2 { x: 0.0, y: 0.0 },
            force: Vector2 { x: 0.0, y: 0.0 },
            mass: 1.0,
            radius: 10.0,
        }
    }
}

const PARTICLE_LIMIT: usize = 1000;
fn new_particle(particles: &mut Vec<Particle>, position: Vector2) -> Option<&mut Particle> {
    for particle in particles.iter_mut() {
        if particle.active {
            continue;
        }

        *particle = Particle::new(position);

        return Some(particle);
    }

    None // no free particle available
}

fn render_particle(d: &mut RaylibDrawHandle, particle: &mut Particle) -> () {
    d.draw_circle_v(particle.position, particle.radius, Color::WHITE)
}

fn main() {
    // Creating empty particle array
    let mut particles = Vec::with_capacity(PARTICLE_LIMIT);
    for i in 0..PARTICLE_LIMIT {
        particles.push(Particle::new(Vector2 { x: 0.0, y: 0.0 }));
        particles[i].active = false;
    }

    // You know what I really like Rust syntax
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Mucky Physics")
        .build();

    let part = new_particle(&mut particles, Vector2 { x: 100.0, y: 100.0 }).unwrap();

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::BLACK);
        render_particle(&mut d, part);
    }
}
