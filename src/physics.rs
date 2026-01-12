use raylib::prelude::*;

pub struct Particle {
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
            pos,
            vel: Vector2 { x: 0.0, y: 0.0 },
            acc: Vector2 { x: 0.0, y: 0.0 },
            force: Vector2 { x: 0.0, y: 0.0 },
            mass: 1.0,
            radius: 6.0,
        }
    }
    pub fn integrate(&mut self, dt: f32) {
        let new_pos = self.pos + self.vel * dt + self.acc * (dt * dt * 0.5);
        let new_acc = self.apply_forces();
        let new_vel = self.vel + (self.acc + new_acc) * (dt * 0.5);
        self.pos = new_pos;
        self.vel = new_vel;
        self.acc = new_acc;
    }
    fn apply_forces(&self) -> Vector2 {
        self.force / self.mass
    }

    pub fn collide(a: &mut Particle, b: &mut Particle) {
        // This has taken me such a long time to figure out...
        // Gaze, at my glorious creation!

        let delta = b.pos - a.pos;
        let dist = delta.length();
        let overlap = (a.radius + b.radius) - dist;

        // Check if there's any collision in the first place
        if overlap > 0.0 {
            let norm = delta / dist;

            // How aligned the velocity is with the normal
            let alignment = (a.vel - b.vel).dot(norm);

            // If alignment is < 0, then particles are separating
            // Otherwise we get some strange collision artifacts...
            if alignment < 0.0 {
                return;
            }

            // Part of the velocity along normal
            let avn = norm * a.vel.dot(norm);
            let bvn = norm * b.vel.dot(norm);

            // Part of the velocity that lies
            // perpendicular to the collision normal
            let avt = a.vel - avn;
            let bvt = b.vel - bvn;

            // Get vel' by swapping vel along normal
            let avelnew = avt + bvn;
            let bvelnew = bvt + avn;

            // We did it!
            a.vel = avelnew;
            b.vel = bvelnew;
        }
    }
}
