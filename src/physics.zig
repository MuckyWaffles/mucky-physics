const rl = @import("raylib");
const std = @import("std");
const math = std.math;

const deltaTime: f32 = 0.01;

const gravity: f32 = 200.0;

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

        // Verlet Integration
        const position = self.position;
        const previous = self.previous;
        const vel = position.subtract(previous);

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
