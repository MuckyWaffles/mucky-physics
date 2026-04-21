use raylib::prelude::*;

// These are the standard values that any
// physics object in the program should hold
#[derive(Clone)]
pub struct Motion {
    pub pos: Vector2,
    pub vel: Vector2,
    pub acc: Vector2,
    pub force: Vector2,
    pub mass: f32,
}

impl Motion {
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
}

// TODO: figure out a way not to
// store position in shapes?
#[derive(Copy, Clone)]
pub struct Circle {
    pub radius: f32,
}
impl Circle {
    pub fn new(radius: f32) -> Circle {
        Circle { radius: radius }
    }
}

#[derive(Copy, Clone)]
pub struct CircleSeg {
    pub radius: f32,
    pub norm: Vector2,
    pub dist: f32,
}

impl CircleSeg {
    pub fn new(radius: f32, norm: Vector2) -> CircleSeg {
        CircleSeg {
            radius: radius,
            norm: norm,
            dist: 0.0,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Polygon {
    pub vertices: Vec<Vector2>,
}

impl Polygon {
    pub fn new() -> Polygon {
        Polygon {}
    }
}

#[derive(Clone)]
pub enum Shape {
    Circle(Circle),
    CircleSeg(CircleSeg),
}

impl Shape {
    pub fn collide(
        shape: &Shape,
        a_pos: Vector2,
        a_vel: Vector2,
        other: &Shape,
        b_pos: Vector2,
        b_vel: Vector2,
    ) -> Option<Vector2> {
        match (shape, other) {
            (Shape::Circle(a), Shape::Circle(b)) => {
                let delta = b_pos - a_pos;
                let dist = delta.length();
                let overlap = (a.radius + b.radius) - dist;

                // Check if there's any collision in the first place
                if overlap > 0.0 {
                    let norm = delta / dist;

                    // How aligned the velocity is with the normal
                    let alignment = (a_vel - b_vel).dot(norm);
                    // If alignment is < 0, then particles are separating
                    if alignment < 0.0 {
                        return None;
                    }

                    return Some(delta.normalized());
                } else {
                    return None;
                }
            }
            (Shape::Circle(a), Shape::CircleSeg(b)) => panic!(),
            (Shape::CircleSeg(a), Shape::Circle(b)) => {
                let delta = a_pos - b_pos;
                let dist = delta.length();
                let overlap = (a.radius + b.radius) - dist;

                // Check if there's any collision in the first place
                if overlap > 0.0 {
                    let norm = delta / dist;

                    // How aligned the velocity is with the normal
                    let alignment = (a_vel - b_vel).dot(norm);
                    // If alignment is < 0, then particles are separating
                    if alignment > 0.0 {
                        return None;
                    }

                    let slope = a.norm.x * (a.norm.y / 1.0);
                    if (b_pos.y - a_pos.y) > slope * (b_pos.x - a_pos.x) {
                        return None;
                    }

                    return Some(delta.normalized());
                } else {
                    return None;
                }
            }
            (Shape::CircleSeg(a), Shape::CircleSeg(b)) => panic!(),
        }
    }
}

#[derive(Clone)]
pub struct Body {
    pub motion: Motion,
    pub shape: Shape,
}

impl Body {
    pub fn new(pos: Vector2, shape: Shape) -> Body {
        Body {
            motion: Motion {
                pos,
                vel: Vector2 { x: 0.0, y: 0.0 },
                acc: Vector2 { x: 0.0, y: 0.0 },
                force: Vector2 { x: 0.0, y: 0.0 },
                mass: 1.0,
            },
            shape: shape,
        }
    }
}

// I'm probably going to rename this function to just "collide" at some point
pub fn collide_with_mass(norm: Vector2, av: &mut Vector2, am: f32, bv: &mut Vector2, bm: f32) {
    let vcm = (*av * am + *bv * bm) / (am + bm);

    let avp = *av - vcm;
    let bvp = *bv - vcm;

    // Part of the velocity along normal
    let avn = norm * avp.dot(norm);
    let bvn = norm * bvp.dot(norm);

    // Part of the velocity that lies
    // perpendicular to the collision normal
    let avt = avp - avn;
    let bvt = bvp - bvn;

    // Get vel' by swapping vel along normal
    let avelnew = avt - avn;
    let bvelnew = bvt - bvn;

    // We did it!
    *av = avelnew + vcm;
    *bv = bvelnew + vcm;
}
