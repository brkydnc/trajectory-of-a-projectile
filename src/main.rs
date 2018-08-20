extern crate opengl_graphics;
extern crate graphics;
extern crate glutin_window;
extern crate piston_window;
extern crate piston;
extern crate vecmath;

use opengl_graphics::{GlGraphics, OpenGL};
use graphics::{Ellipse, clear};
use glutin_window::*;
use piston_window::{MouseButton, Window, WindowSettings, ellipse::circle};
use piston::{event_loop::*, input::*};
use vecmath::*;
use std::f64;

struct App {
    // Dependencies
    events: Events,
    window: GlutinWindow,
    opengl: GlGraphics,
    // Options & stuff
    mouse_position: Vector2<f64>,
    gravity: Vector2<f64>,
    velocity_divide_coefficient: Vector2<f64>,
    trajectory_dot_count: u32,
    // Main objects
    launcher: Vector2<f64>,
    balls: Vec<Ball>,
    trajectory: Vec<[f64; 2]>,
}

#[derive(Copy, Clone, PartialEq, Debug)]
struct Ball {
    position: Vector2<f64>,
    velocity: Vector2<f64>,
    radius: f64,
}

impl App {
    fn render(&mut self, args: &RenderArgs) {
        let (launcher_x, launcher_y): (f64, f64) = (self.launcher[0], self.launcher[1]);
        let balls: &mut Vec<Ball> = &mut self.balls;
        let trajectory: &Vec<[f64; 2]> = &self.trajectory;

        self.opengl.draw(args.viewport(), |_context, _graphics| {
            // Background
            clear([0.0, 0.0, 0.0, 1.0], _graphics);

            // Launcher dot
            Ellipse::new([0.0, 1.0, 0.0, 1.0]).draw(
                // Green
                circle(launcher_x, launcher_y, 5.0),
                &_context.draw_state,
                _context.transform,
                _graphics,
            );

            // Trajectory
            for displacement in trajectory {
                let (displacement_x, displacement_y) =
                    (displacement[0] + launcher_x, displacement[1] + launcher_y);

                Ellipse::new([1.0, 0.0, 0.0, 0.5]).draw(
                    // 50% Red
                    circle(displacement_x, displacement_y, 3.0),
                    &_context.draw_state,
                    _context.transform,
                    _graphics,
                );
            }

            // Balls
            for ball in balls {
                Ellipse::new([1.0, 1.0, 1.0, 1.0]).draw(
                    // White
                    circle(ball.position[0], ball.position[1], ball.radius),
                    &_context.draw_state,
                    _context.transform,
                    _graphics,
                );
            }
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        let (width, height): (f64, f64) = (
            self.window.size().width as f64,
            self.window.size().height as f64,
        );

        self.balls.retain(|&b| {
            b.position[0] - b.radius < width
                && b.position[0] + b.radius > 0.0
                && b.position[1] - b.radius < height
        });

        for ball in &mut self.balls {
            ball.update(args.dt * 10.0, self.gravity);
        }
    }

    fn update_mouse_position(&mut self, args: [f64; 2]) {
        self.mouse_position = args;
    }

    fn update_launcher_position(&mut self) {
        self.launcher = self.mouse_position;
    }

    fn update_trajectory_dot_count(&mut self, args: [f64; 2]) {
        if args[1] < 0.0 {
            self.trajectory_dot_count += 1;
        } else if self.trajectory_dot_count != 0 {
            self.trajectory_dot_count -= 1
        }
    }

    fn calculate_trajectory(&mut self) {
        let launch_velocity: Vector2<f64> = vec2_mul(
            vec2_sub(self.mouse_position, self.launcher),
            self.velocity_divide_coefficient,
        );
        let velocity_mag: f64 = vec2_len(launch_velocity);
        let launch_angle: f64 = (launch_velocity[1]).atan2(launch_velocity[0]);

        self.trajectory = Vec::new();
        for t in 1..self.trajectory_dot_count + 1 {
            let x = velocity_mag * t as f64 * launch_angle.cos();
            let y = velocity_mag * t as f64 * launch_angle.sin()
                - 0.5 * (-self.gravity[1]) * (t as f64).powf(2.0);
            let displacement = [x, y];
            self.trajectory.push(displacement);
        }
    }

    fn launch_ball(&mut self) {
        let launch_velocity: Vector2<f64> = vec2_mul(
            vec2_sub(self.mouse_position, self.launcher),
            self.velocity_divide_coefficient,
        );
        self.balls.push(Ball::new(
            self.launcher[0],
            self.launcher[1],
            launch_velocity[0],
            launch_velocity[1],
            8.0,
        ));
    }
}

impl Ball {
    fn new(x: f64, y: f64, vx: f64, vy: f64, r: f64) -> Self {
        Ball {
            position: [x, y],
            velocity: [vx, vy],
            radius: r,
        }
    }

    fn update(&mut self, delta_time: f64, gravity: Vector2<f64>) {
        self.velocity = vec2_add(self.velocity, vec2_mul(gravity, [delta_time; 2]));
        self.position = vec2_add(self.position, vec2_mul(self.velocity, [delta_time; 2]));
    }
}

fn main() {
    let mut app: App = App {
        events: Events::new(EventSettings::new()),
        window: WindowSettings::new("Trajectory of a Projectile", [900, 500])
            .opengl(OpenGL::V4_5)
            .vsync(true)
            .exit_on_esc(true)
            .build()
            .unwrap(),
        opengl: GlGraphics::new(OpenGL::V4_5),

        mouse_position: [0.0, 0.0],
        gravity: [0.0, 9.8],
        velocity_divide_coefficient: [1.0 / 2.5; 2],
        trajectory_dot_count: 10,

        launcher: [20.0, 250.0],
        balls: Vec::new(),
        trajectory: Vec::new(),
    };

    while let Some(e) = app.events.next(&mut app.window) {
        // Update
        if let Some(update_args) = e.update_args() {
            app.update(&update_args);
        }

        // Render
        if let Some(render_args) = e.render_args() {
            app.render(&render_args);
        }

        // Mouse Cursor Event (Move)
        if let Some(position) = e.mouse_cursor_args() {
            app.update_mouse_position(position);
            app.calculate_trajectory();
        }

        // Press Event
        if let Some(Button::Mouse(mouse_button)) = e.press_args() {
            match mouse_button {
                MouseButton::Left => {
                    app.launch_ball();
                }
                MouseButton::Right => {
                    app.update_launcher_position();
                }
                _ => {}
            }
        }

        // Mouse Scroll Event
        if let Some(scroll_args) = e.mouse_scroll_args() {
            app.update_trajectory_dot_count(scroll_args);
            app.calculate_trajectory();
        }
    }
}