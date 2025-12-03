const rl = @import("raylib");
const std = @import("std");
const math = std.math;

const deltaTime: f32 = 0.01;

const gravity: f32 = 200.0;

const particleLimit: u16 = 600;
pub var particles: [particleLimit]Particle = undefined;
pub var particlesLeft: [particleLimit]Particle = undefined;
pub var particlesRight: [particleLimit]Particle = undefined;

const constraintLimit: u16 = 20;
pub var constraints: [constraintLimit]Constraint = undefined;

const edgeLimit: u16 = 20;
pub var edges: [edgeLimit]Edge = undefined;

pub const Options = struct {
    pub var gravity = false;
    pub var edges = false;
    pub var drag = false;
};

/// Collide particles of equal mass
pub fn collideParticles(b: *Particle, a: *Particle) void {
    const v1 = a.position.subtract(a.previous);
    const v2 = b.position.subtract(b.previous);

    const n = rl.Vector2.subtract(a.position, b.position).normalize();

    // How aligned the velocity is with the normal
    const rel = v1.subtract(v2);
    const alignment = rel.dotProduct(n);

    // if alignment is > 0, then particles are moving away
    if (alignment > 0) return;

    // 3. Apply velocity swap along the normal
    const v1p = v1.subtract(n.scale(alignment));
    const v2p = v2.add(n.scale(alignment));

    // 4. Update previous positions for Verlet
    a.previous = a.position.subtract(v1p);
    b.previous = b.position.subtract(v2p);
}

/// Particle with Verlet Integration
pub const Particle = struct {
    inUse: bool = false,

    previous: rl.Vector2 = rl.Vector2{ .x = 0, .y = 0 },
    position: rl.Vector2 = rl.Vector2{ .x = 0, .y = 0 },
    accel: rl.Vector2 = rl.Vector2{ .x = 0, .y = 0 },
    force: rl.Vector2 = rl.Vector2{ .x = 0, .y = 0 },
    mass: f32 = 1.0,
    radius: f32 = 10,

    /// Update using Verlet integration
    pub fn integrate(self: *Particle) void {
        if (Options.gravity) self.force.y += gravity * self.mass;

        const position = self.position;
        const previous = self.previous;
        var vel = position.subtract(previous);

        if (Options.drag) {
            const velMag = std.math.hypot(vel.x, vel.y);
            const drag = 0.001 * 0.5 * velMag * velMag;
            vel = rl.Vector2.scale(vel.normalize(), velMag - drag);
        }

        self.accel = self.accel.add(self.force.scale(1 / self.mass));
        self.force = rl.Vector2{ .x = 0, .y = 0 };

        self.position.x += vel.x + self.accel.x * math.pow(f32, deltaTime, 2);
        self.position.y += vel.y + self.accel.y * math.pow(f32, deltaTime, 2);
        self.previous = position;

        self.accel = rl.Vector2{ .x = 0, .y = 0 };
    }

    pub fn collide(self: *Particle) void {
        // Collision Strength
        const strength = 1.00;

        // Main particles
        for (&particles) |*particle| {
            if (!particle.inUse) continue;

            // const length = self.radius + particle.radius;
            // const delta = self.position.subtract(particle.position);
            // const deltaLength = math.hypot(delta.x, delta.y);
            // if (deltaLength > 0 and deltaLength < length) {
            // const diff = (deltaLength - length) / deltaLength;
            // if (math.isNan(diff)) return;

            // const startAlpha = 1.0 / particle.mass;
            // const endAlpha = 1.0 / self.mass;

            // particle.position.x += delta.x * diff * strength * startAlpha;
            // particle.position.y += delta.y * diff * strength * startAlpha;
            // self.position.x -= delta.x * diff * strength * endAlpha;
            // self.position.y -= delta.y * diff * strength * endAlpha;

            // }

            collideParticles(self, particle);
        }
        // Left images
        for (&particlesLeft) |*particle| {
            if (!particle.inUse) continue;

            // const length = self.radius + particle.radius;
            // const delta = self.position.subtract(particle.position);
            // const deltaLength = math.hypot(delta.x, delta.y);
            // if (deltaLength > 0 and deltaLength < length) {
            // const diff = (deltaLength - length) / deltaLength;
            // if (math.isNan(diff)) return;

            // const startAlpha = 1.0 / particle.mass;
            // const endAlpha = 1.0 / self.mass;

            // particle.position.x += delta.x * diff * strength * startAlpha;
            // particle.position.y += delta.y * diff * strength * startAlpha;
            // self.position.x -= delta.x * diff * strength * endAlpha;
            // self.position.y -= delta.y * diff * strength * endAlpha;
            // }

            collideParticles(self, particle);
        }
        // Left images
        for (&particlesRight) |*particle| {
            if (!particle.inUse) continue;

            // const length = self.radius + particle.radius;
            // const delta = self.position.subtract(particle.position);
            // const deltaLength = math.hypot(delta.x, delta.y);
            // if (deltaLength > 0 and deltaLength < length) {
            // const diff = (deltaLength - length) / deltaLength;
            // if (math.isNan(diff)) return;

            // const startAlpha = 1.0 / particle.mass;
            // const endAlpha = 1.0 / self.mass;

            // particle.position.x += delta.x * diff * strength * startAlpha;
            // particle.position.y += delta.y * diff * strength * startAlpha;
            // self.position.x -= delta.x * diff * strength * endAlpha;
            // self.position.y -= delta.y * diff * strength * endAlpha;
            // }

            collideParticles(self, particle);
        }

        if (!Options.edges) return;
        for (edges) |edge| {
            var contact: rl.Vector2 = undefined;
            if (getLineIntersection(
                edge.start,
                edge.end,
                self.position.add(edge.normal.scale(16)),
                self.position,
                &contact,
            )) {
                const power = self.position.distance(contact);
                self.position.x += edge.normal.x * power;
                self.position.y += edge.normal.y * power;
            }
        }
    }
};

pub const ConstraintType = enum {
    both,
    push,
    pull,
};

fn getLineIntersection(p0: rl.Vector2, p1: rl.Vector2, p2: rl.Vector2, p3: rl.Vector2, i: ?*rl.Vector2) bool {
    const s1: rl.Vector2 = p1.subtract(p0);
    const s2: rl.Vector2 = p3.subtract(p2);

    const s = (-s1.y * (p0.x - p2.x) + s1.x * (p0.y - p2.y)) / (-s2.x * s1.y + s1.x * s2.y);
    const t = (s2.x * (p0.y - p2.y) - s2.y * (p0.x - p2.x)) / (-s2.x * s1.y + s1.x * s2.y);

    if (s >= 0 and s <= 1 and t >= 0 and t <= 1) {
        // Collision detected
        if (i != null) {
            i.?.x = p0.x + (t * s1.x);
            i.?.y = p0.y + (t * s1.y);
        }
        return true;
    }

    return false; // No collision
}

pub const Edge = struct {
    inUse: bool = false,

    start: rl.Vector2,
    end: rl.Vector2,
    normal: rl.Vector2,
};

pub const Constraint = struct {
    inUse: bool = false,

    type: ConstraintType,
    start: *Particle,
    end: *Particle,
    length: f32,
    strength: f32,

    pub fn satisfy(self: *Constraint) void {
        const start = self.start;
        const end = self.end;
        const strength = self.strength;

        const delta = end.position.subtract(start.position);
        const deltaLength = math.hypot(delta.x, delta.y);

        if (self.type == ConstraintType.push) {
            if (deltaLength > self.length) return;
        } else if (self.type == ConstraintType.pull) {
            if (deltaLength < self.length) return;
        }

        if (deltaLength > 0) {
            const diff = (deltaLength - self.length) / deltaLength;
            if (math.isNan(diff)) return;

            const startAlpha = 1.0 / start.mass;
            const endAlpha = 1.0 / end.mass;

            start.position.x += delta.x * diff * strength * startAlpha;
            start.position.y += delta.y * diff * strength * startAlpha;
            end.position.x -= delta.x * diff * strength * endAlpha;
            end.position.y -= delta.y * diff * strength * endAlpha;
        }
    }
};
