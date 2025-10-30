const rl = @import("raylib");
const std = @import("std");
const math = std.math;

const deltaTime: f32 = 0.01;

const gravity: f32 = 200.0;

const particleLimit: u16 = 600;
pub var particles: [particleLimit]Particle = undefined;

/// Particle with Verlet Integration
pub const Particle = struct {
    inUse: bool = false,

    previous: rl.Vector2 = rl.Vector2{ .x = 0, .y = 0 },
    position: rl.Vector2 = rl.Vector2{ .x = 0, .y = 0 },
    accel: rl.Vector2 = rl.Vector2{ .x = 0, .y = 0 },
    mass: f32 = 1.0,
    radius: f32 = 10,

    /// Update using Verlet integration
    pub fn update(self: *Particle) void {
        self.accel.y += gravity;

        for (&particles) |*particle| {
            if (!particle.inUse) continue;
            if (particle.position.x == self.position.x and
                particle.position.y == self.position.y)
            {
                continue;
            }
            const length = self.radius + particle.radius;
            const delta = self.position.subtract(particle.position);
            const deltaLength = math.hypot(delta.x, delta.y);
            if (deltaLength > 0 and deltaLength < length) {
                const diff = (deltaLength - length) / deltaLength;
                if (math.isNan(diff)) return;

                const startAlpha = 1.0 / particle.mass;
                const endAlpha = 1.0 / self.mass;
                const strength = 0.10;

                particle.position.x += delta.x * diff * strength * startAlpha;
                particle.position.y += delta.y * diff * strength * startAlpha;
                self.position.x -= delta.x * diff * strength * endAlpha;
                self.position.y -= delta.y * diff * strength * endAlpha;
            }
        }

        // Verlet Integration
        const position = self.position;
        const previous = self.previous;
        var vel = position.subtract(previous);

        const velMag = std.math.hypot(vel.x, vel.y);
        const drag = 0.001 * 0.5 * velMag * velMag;
        vel = rl.Vector2.scale(vel.normalize(), velMag - drag);

        self.position.x += vel.x + self.accel.x * math.pow(f32, deltaTime, 2);
        self.position.y += vel.y + self.accel.y * math.pow(f32, deltaTime, 2);
        self.previous = position;

        self.accel = rl.Vector2{ .x = 0, .y = 0 };
    }
};

pub const ConstraintType = enum {
    both,
    push,
    pull,
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
