use raylib::prelude::*;

pub struct Particle {
    active: bool,

    pub pos: Vector2,
    pub vel: Vector2,
    pub acc: Vector2,
    pub force: Vector2,
    pub mass: f32,
    pub radius: f32,
}

impl Particle {
    pub fn new(pos: Vector2) -> Particle {
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
    pub fn integrate(&mut self, dt: f32) {
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

    pub fn is_active(&self) -> bool {
        self.active
    }
    pub fn end(&mut self) {
        self.active = false;
    }
}
