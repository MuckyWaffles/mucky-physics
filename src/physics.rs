use rand::Rng;
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

#[derive(Clone)]
pub struct Polygon {
    pub vertices: Vec<Vector2>,
}

impl Polygon {
    pub fn new(vertices: Vec<Vector2>) -> Polygon {
        Polygon { vertices }
    }
}

#[derive(Clone)]
pub enum Shape {
    Circle(Circle),
    Polygon(Polygon),
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
            (Shape::Circle(a), Shape::Polygon(b)) => panic!(),
            (Shape::Polygon(a), Shape::Circle(b)) => {
                let mut intersection = 0;
                let mut intersections = 0;
                let len = a.vertices.len();

                for i in 0..len {
                    let seg = (a.vertices[(i + 1) % len] - a.vertices[i]).normalized();
                    let norm = Vector2 {
                        x: seg.y,
                        y: -seg.x,
                    };
                    let point = b_pos + norm * b.radius;
                    let point_start = point - (b_vel - a_vel);

                    if line_intersect(
                        a.vertices[i],
                        a.vertices[(i + 1) % len],
                        point_start - a_pos,
                        point - a_pos,
                    ) {
                        intersection = i;
                        intersections += 1;
                    }
                }

                let inside = intersections % 2 == 1;
                if intersections == 0 {
                    return None;
                }

                let i = intersection;
                let seg = (a.vertices[(i + 1) % len] - a.vertices[i]).normalized();
                let mut norm = Vector2 {
                    x: seg.y,
                    y: -seg.x,
                };
                if point_in_polygon(a.vertices.clone(), seg, Vector2 { x: 0.0, y: 0.0 }) {
                    norm = -norm;
                }
                return Some(norm);
            }
            (Shape::Polygon(a), Shape::Polygon(b)) => panic!(),
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

fn line_intersect(a0: Vector2, a1: Vector2, b0: Vector2, b1: Vector2) -> bool {
    let sa = a1 - a0;
    let sb = b1 - b0;
    let s = (-sa.y * (a0.x - b0.x) + sa.x * (a0.y - b0.y)) / (-sb.x * sa.y + sa.x * sb.y);
    let t = (sb.x * (a0.y - b0.y) - sb.y * (a0.x - b0.x)) / (-sb.x * sa.y + sa.x * sb.y);
    return s >= 0.0 && s <= 1.0 && t >= 0.0 && t <= 1.0;
}

fn point_in_polygon(polygon: Vec<Vector2>, point: Vector2, start: Vector2) -> bool {
    let mut intersections = 0;
    let len = polygon.len();

    for i in 0..len {
        if line_intersect(polygon[i], polygon[(i + 1) % len], start, point) {
            intersections += 1;
        }
    }

    return intersections % 2 == 1;
}
