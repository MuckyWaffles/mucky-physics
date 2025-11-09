// raylib-zig (c) Nikolas Wipper 2023

const std = @import("std");
const math = std.math;

// P for physics, because why wouldn't I choose
// the shortest possible name for it?
const p = @import("physics.zig");
const rl = @import("raylib");
const rg = @import("raygui");

const screenWidth = 1000;
const screenHeight = 600;

/// Render the particle, and a ring around it to show where it can be grabbed
fn renderParticle(self: *p.Particle) void {
    // Here we draw two things:
    //rl.drawCircleLinesV(self.position, self.radius + 6, .red);
    rl.drawCircleV(self.position, self.radius, .white);

    if (showMass) {
        rl.drawText(
            rl.textFormat("%.2f", .{self.mass - @mod(self.mass, 0.01)}),
            @intFromFloat(self.position.x),
            @intFromFloat(self.position.y + 10),
            18,
            .light_gray,
        );
    }
}

fn newParticle(position: rl.Vector2, mass: f32) error{NoSpace}!*p.Particle {
    for (&p.particles) |*particle| {
        if (particle.*.inUse) continue;

        particle.* = p.Particle{
            .inUse = true,
            .position = position,
            .previous = position,
            .accel = rl.Vector2{ .x = 0, .y = 0 },
            .mass = mass,
        };
        return particle;
    }
    return error.NoSpace;
}

fn constrainParticleToScreen(particle: *p.Particle) void {
    const velocity = particle.position.subtract(particle.previous);

    if (particle.position.y + particle.radius > screenHeight) {
        particle.position.y = screenHeight - particle.radius;
        particle.previous.y = particle.position.y + velocity.y;
    }
    if (particle.position.y - particle.radius < 0) {
        particle.position.y = particle.radius;
        particle.previous.y = particle.position.y + velocity.y;
    }
    if (particle.position.x + particle.radius > screenWidth) {
        particle.position.x = screenWidth - particle.radius;
        particle.previous.x = particle.position.x + velocity.x;
    }
    if (particle.position.x - particle.radius < 0) {
        particle.position.x = particle.radius;
        particle.previous.x = particle.position.x + velocity.x;
    }
}

fn newConstraint(
    start: *p.Particle,
    end: *p.Particle,
    length: f32,
    strength: f32,
    constraintType: p.ConstraintType,
) error{NoSpace}!*p.Constraint {
    for (&p.constraints) |*constraint| {
        if (constraint.*.inUse) continue;

        constraint.* = p.Constraint{
            .inUse = true,
            .start = start,
            .end = end,
            .length = length,
            .strength = strength,
            .type = constraintType,
        };
        return constraint;
    }
    return error.NoSpace;
}

fn newEdge(
    start: rl.Vector2,
    end: rl.Vector2,
) error{NoSpace}!*p.Edge {
    for (&p.edges) |*edge| {
        if (edge.*.inUse) continue;

        const norm = rl.Vector2{ .x = 0, .y = -1 };

        edge.* = p.Edge{
            .inUse = true,
            .start = start,
            .end = end,
            .normal = norm,
        };
        return edge;
    }
    return error.NoSpace;
}

fn renderConstraint(self: *p.Constraint) void {
    rl.drawLineEx(self.start.*.position, self.end.*.position, 2.0, .purple);
}

fn renderEdge(self: *p.Edge) void {
    rl.drawLineEx(self.start, self.end, 2.0, .green);
}

fn physics_process() !void {

    // Creating particles and constraints
    const mouseParticle = try newParticle(rl.getMousePosition(), 9999);

    const p1 = try newParticle(rl.Vector2{ .x = 100, .y = 200 }, 1.0);
    const p2 = try newParticle(rl.Vector2{ .x = 300, .y = 250 }, 2.0);
    _ = try newConstraint(p1, p2, 100.0, 0.0005, .pull);

    const s1 = try newParticle(rl.Vector2{ .x = 200, .y = 200 }, 1.0);
    const s2 = try newParticle(rl.Vector2{ .x = 300, .y = 200 }, 1.0);
    const s3 = try newParticle(rl.Vector2{ .x = 300, .y = 300 }, 1.0);
    const s4 = try newParticle(rl.Vector2{ .x = 200, .y = 300 }, 1.0);
    const boxStrength = 0.05;
    _ = try newConstraint(s1, s2, 100.0, boxStrength, .both);
    _ = try newConstraint(s2, s3, 100.0, boxStrength, .both);
    _ = try newConstraint(s3, s4, 100.0, boxStrength, .both);
    _ = try newConstraint(s4, s1, 100.0, boxStrength, .both);
    _ = try newConstraint(s1, s3, math.hypot(100.0, 100.0), boxStrength, .both);
    _ = try newConstraint(s4, s2, math.hypot(100.0, 100.0), boxStrength, .both);

    _ = try newEdge(
        rl.Vector2{ .x = 0, .y = screenHeight - 40 },
        rl.Vector2{ .x = screenWidth / 2, .y = screenHeight - 40 },
    );
    _ = try newEdge(
        rl.Vector2{ .x = screenWidth / 2, .y = screenHeight - 40 },
        rl.Vector2{ .x = screenWidth, .y = screenHeight - 40 },
    );

    for (0..400) |_| {
        //const spawnX: f32 = @floatFromInt(rl.getRandomValue(0, @intFromFloat(screenWidth)));
        //const spawnY: f32 = @floatFromInt(rl.getRandomValue(50, @intFromFloat(screenHeight)));
        //_ = try newParticle(rl.Vector2{ .x = spawnX, .y = spawnY }, 1.0);
    }

    // Creating particle that follows mouse
    const mouseDrag = try newConstraint(mouseParticle, s1, 0.0, 0.002, .both);

    // Physics loop
    while (!rl.windowShouldClose()) {
        const startTime = rl.getTime();

        for (&p.particles) |*particle| {
            if (!particle.inUse) continue;
            particle.update();
            constrainParticleToScreen(particle);
        }
        for (&p.constraints) |*constraint| {
            if (!constraint.inUse) continue;
            constraint.satisfy();
        }

        mouseParticle.position = rl.getMousePosition();
        mouseDrag.strength = 0.0;
        if (rl.isMouseButtonDown(rl.MouseButton.left)) {
            for (&p.particles) |*particle| {
                if (!particle.inUse) continue;
                const delta = rl.getMousePosition().subtract(particle.position);
                const dist = math.hypot(delta.x, delta.y);

                mouseDrag.strength = 0.002;
                _ = dist;
                //if (dist > particle.radius + 6) continue;
            }
        }

        const endTime = rl.getTime();
        const frameTime = endTime - startTime;

        // Maintaining a frame length of exactly 0.01
        rl.waitTime(0.01 - frameTime);

        // TODO: we should probably check if we go under 0,
        // But this loop is quick enough that we shouldn't
        // need to worry about such things
    }
}

var physicsThread: std.Thread = undefined;

pub fn main() anyerror!void {
    // Initialization
    rl.initWindow(screenWidth, screenHeight, "Mucky Physics");
    defer rl.closeWindow(); // Close window and OpenGL context

    rl.setTargetFPS(60); // Set our game to run at 60 frames-per-second

    physicsThread = try std.Thread.spawn(.{}, physics_process, .{});

    // Main game loop
    while (!rl.windowShouldClose()) {
        rl.beginDrawing();
        defer rl.endDrawing();

        rl.clearBackground(.black);

        for (&p.particles) |*particle| {
            if (!particle.inUse) continue;
            renderParticle(particle);
        }
        for (&p.constraints) |*constraint| {
            if (!constraint.inUse) continue;
            renderConstraint(constraint);
        }
        for (&p.edges) |*edge| {
            if (!edge.inUse) continue;
            renderEdge(edge);
        }
        drawUI();
    }
}

var enableUI = false;
var showMass = false;
fn drawUI() void {
    if (rg.button(rl.Rectangle{
        .x = 10,
        .y = 10,
        .width = 60,
        .height = 24,
    }, "Toggle UI")) {
        enableUI = !enableUI;
    }

    // Exit if UI isn't enabled
    if (!enableUI) return;

    const windowRect = rl.Rectangle{
        .x = screenWidth - 220,
        .y = 10,
        .width = 200,
        .height = 400,
    };
    _ = rg.windowBox(windowRect, "Menu");
    _ = rg.checkBox(
        rl.Rectangle{
            .x = windowRect.x + 10,
            .y = windowRect.y + 35,
            .width = 60,
            .height = 20,
        },
        "Show Mass",
        &showMass,
    );
}
