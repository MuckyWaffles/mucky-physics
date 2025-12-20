use raylib::prelude::*;

const SCREEN_WIDTH: i32 = 1000;
const SCREEN_HEIGHT: i32 = 600;

struct Particle {
    active: bool,

    pos: Vector2,
    vel: Vector2,
    acc: Vector2,
    force: Vector2,
    mass: f32,
    radius: f32,
}

impl Particle {
    fn new(pos: Vector2) -> Particle {
        Particle {
            active: true,
            pos,
            vel: Vector2 { x: 0.0, y: 0.0 },
            acc: Vector2 { x: 0.0, y: 0.0 },
            force: Vector2 { x: 0.0, y: 0.0 },
            mass: 1.0,
            radius: 10.0,
        }
    }
    fn integrate(&mut self, dt: f32) {
        let new_pos = self.pos + self.vel * dt + self.acc * (dt * dt * 0.5);
        let new_acc = self.acc + self.apply_forces();
        let new_vel = self.vel + new_acc * (dt * 0.5);
        self.pos = new_pos;
        self.vel = new_vel;
        self.acc = new_acc;
    }
    fn apply_forces(&self) -> Vector2 {
        self.force / self.mass
    }
}

const PARTICLE_LIMIT: usize = 1000;
fn new_particle(particles: &mut Vec<Particle>, pos: Vector2) -> Option<&mut Particle> {
    for particle in particles.iter_mut() {
        if particle.active {
            continue;
        }

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
        particles[i].active = false;
    }

    // You know what I really like Rust syntax
    let (mut rl, thread) = raylib::init()
        .msaa_4x()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Mucky Physics")
        .build();

    rl.set_target_fps(60);
    let dt = 1.0 / 60.0;

    let _ = new_particle(&mut particles, Vector2 { x: 100.0, y: 100.0 }).unwrap();

    while !rl.window_should_close() {
        for particle in particles.iter_mut() {
            if !particle.active {
                continue;
            };
            particle.force.y += GRAVITY * particle.mass;
            particle.integrate(dt);
        }

        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::BLACK);
        for particle in particles.iter_mut() {
            if !particle.active {
                continue;
            };
            render_particle(&mut d, particle);
        }
    }
}
