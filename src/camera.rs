use nalgebra::{Matrix4, Point3, Vector3};
use winit::event::{
    DeviceEvent, KeyboardInput, MouseScrollDelta, VirtualKeyCode,
};

use crate::Graphics;

#[rustfmt::skip]
const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

pub struct Camera {
    pub eye: Point3<f32>,
    pub target: Point3<f32>,
    up: Vector3<f32>,
    pub aspect: f32,
    pub fov: f32,
    near: f32,
    far: f32,
    pub controller: CameraController,
}

impl Camera {
    pub fn new(graphics: &Graphics) -> Self {
        let controller = CameraController::new();
        Self {
            eye: Point3::new(0., 0., 1.),
            target: Point3::new(0., 0., -1.),
            up: Vector3::y(),
            aspect: graphics.config.width as f32
                / graphics.config.height as f32,
            fov: 60.,
            near: 0.01,
            far: 100.0,
            controller,
        }
    }

    pub fn update_global_matrix(&mut self) -> Matrix4<f32> {
        let target = Point3::new(
            self.eye.x + self.target.x,
            self.eye.y + self.target.y,
            self.eye.z + self.target.z,
        );
        let projection = Matrix4::new_perspective(
            self.aspect,
            self.fov.to_degrees(),
            self.near,
            self.far,
        );
        let view = Matrix4::look_at_rh(&self.eye, &target, &self.up);
        OPENGL_TO_WGPU_MATRIX * projection * view
    }

    pub fn resize(&mut self, graphics: &Graphics) {
        self.aspect =
            graphics.config.width as f32 / graphics.config.height as f32;
    }

    pub fn update(&mut self) {
        self.fov += self.controller.fov_delta;
        self.controller.fov_delta = 0.;
        self.target = Point3::new(
            self.controller.yaw.to_radians().cos()
                * self.controller.pitch.to_radians().cos(),
            self.controller.pitch.to_radians().sin(),
            self.controller.yaw.to_radians().sin()
                * self.controller.pitch.to_radians().cos(),
        );
        let target =
            Vector3::new(self.target.x, 0.0, self.target.z).normalize();
        self.eye += &target
            * self.controller.speed
            * (self.controller.forward - self.controller.backward);
        self.eye += &target.cross(&self.up)
            * self.controller.speed
            * (self.controller.right - self.controller.left);
        self.eye += Vector3::new(0.0, 1.0, 0.0)
            * self.controller.speed
            * (self.controller.up - self.controller.down);
    }

    pub fn input(&mut self, event: &winit::event::DeviceEvent) {
        self.controller.process_input(event);
    }
}

pub struct CameraController {
    speed: f32,
    sensitivity: f64,
    forward: f32,
    backward: f32,
    left: f32,
    right: f32,
    up: f32,
    down: f32,
    pub yaw: f32,
    pub pitch: f32,
    fov_delta: f32,
}

impl CameraController {
    pub fn new() -> Self {
        CameraController {
            speed: 0.08,
            sensitivity: 0.1,
            forward: 0.0,
            backward: 0.0,
            left: 0.0,
            right: 0.0,
            up: 0.0,
            down: 0.0,
            yaw: 270.0,
            pitch: 0.0,
            fov_delta: 0.0,
        }
    }

    pub fn process_input(&mut self, event: &winit::event::DeviceEvent) {
        match event {
            /*DeviceEvent::MouseMotion { delta } => {
                self.yaw += (delta.0 * self.sensitivity) as f32;
                self.pitch -= (delta.1 * self.sensitivity) as f32;

                if self.pitch > 89.0 {
                    self.pitch = 89.0;
                } else if self.pitch < -89.0 {
                    self.pitch = -89.0;
                }

                if self.yaw > 360.0 {
                    self.yaw = 0.0;
                } else if self.yaw < 0.0 {
                    self.yaw = 360.0;
                }
            }*/
            DeviceEvent::MouseWheel { delta } => {
                self.fov_delta = match delta {
                    MouseScrollDelta::LineDelta(_, scroll) => *scroll,
                    MouseScrollDelta::PixelDelta(
                        winit::dpi::PhysicalPosition { y, .. },
                    ) => *y as f32,
                }
            }
            DeviceEvent::Motion { .. } => {}
            DeviceEvent::Button { .. } => {}
            DeviceEvent::Key(KeyboardInput {
                state,
                virtual_keycode,
                ..
            }) => {
                let value: f32;
                if *state == winit::event::ElementState::Pressed {
                    value = 1.
                } else {
                    value = 0.;
                }
                match virtual_keycode.unwrap() {
                    VirtualKeyCode::Space => {
                        self.up = value;
                    }
                    VirtualKeyCode::LShift => {
                        self.down = value;
                    }
                    VirtualKeyCode::W => {
                        self.forward = value;
                    }
                    VirtualKeyCode::S => {
                        self.backward = value;
                    }
                    VirtualKeyCode::A => {
                        self.left = value;
                    }
                    VirtualKeyCode::D => {
                        self.right = value;
                    }
                    _ => (),
                }
            }
            _ => (),
        }
    }
}
