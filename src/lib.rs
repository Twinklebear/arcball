#![cfg_attr(feature = "unstable", feature(plugin))]
#![cfg_attr(feature = "unstable", plugin(clippy))]

//! An implementation of the [Shoemake Arcball Camera](http://dl.acm.org/citation.cfm?id=155312)
//! using [cgmath](https://crates.io/crates/cgmath). See the
//! [cube example](https://github.com/Twinklebear/arcball/blob/master/examples/cube.rs) for an example
//! of use with [glium](https://crates.io/crates/glium).
extern crate cgmath;

use cgmath::prelude::*;
use cgmath::{Matrix4, Quaternion, Vector2, Vector3};

/// The Shoemake Arcball camera.
pub struct ArcballCamera {
    look_at: Matrix4<f32>,
    translation: Matrix4<f32>,
    rotation: Quaternion<f32>,
    camera: Matrix4<f32>,
    inv_camera: Matrix4<f32>,
    motion_speed: f32,
    zoom_speed: f32,
    inv_screen: [f32; 2],
}

impl ArcballCamera {
    /// Create a new Arcball camera starting from the look at matrix `look_at`. The `motion_speed`
    /// sets the speed for panning and `zoom_speed` the speed for zooming the camera. `screen` should be
    /// `[screen_width, screen_height]`
    pub fn new(look_at: &Matrix4<f32>, motion_speed: f32, zoom_speed: f32, screen: [f32; 2]) -> ArcballCamera {
        ArcballCamera {
            look_at: look_at.clone(),
            translation: Transform::one(),
            rotation: Quaternion::new(1.0, 0.0, 0.0, 0.0),
            camera: look_at.clone(),
            inv_camera: look_at.invert().unwrap(),
            motion_speed: motion_speed,
            zoom_speed: zoom_speed,
            inv_screen: [1.0 / screen[0], 1.0 / screen[1]],
        }
    }
    /// Get the view matrix computed by the camera.
    pub fn get_mat4(&self) -> Matrix4<f32> {
        self.camera
    }
    /// Rotate the camera, mouse positions should be in pixel coordinates.
    ///
    /// Rotates starting from the orientation at the previous mouse position, `mouse_prev`,
    /// and rotating to desired orientation at the current mouse position, `mouse_cur`.
    pub fn rotate(&mut self, mouse_prev: Vector2<f32>, mouse_cur: Vector2<f32>) {
        let m_cur = Vector2::new(clamp(mouse_cur.x * 2.0 * self.inv_screen[0] - 1.0, -1.0, 1.0),
                                    clamp(1.0 - 2.0 * mouse_cur.y * self.inv_screen[1], -1.0, 1.0));
        let m_prev = Vector2::new(clamp(mouse_prev.x * 2.0 * self.inv_screen[0] - 1.0, -1.0, 1.0),
                                    clamp(1.0 - 2.0 * mouse_prev.y * self.inv_screen[1], -1.0, 1.0));
        let mouse_cur_ball = ArcballCamera::screen_to_arcball(m_cur);
        let mouse_prev_ball = ArcballCamera::screen_to_arcball(m_prev);
        self.rotation = mouse_cur_ball * mouse_prev_ball * self.rotation;
        self.camera = self.translation * self.look_at * Matrix4::from(self.rotation);
        self.inv_camera = self.camera.invert().unwrap();
    }
    /// Zoom the camera in by some amount, positive values zoom in, negative zoom out.
    pub fn zoom(&mut self, amount: f32, elapsed: f32) {
        let motion = Vector3::new(0.0, 0.0, amount);
        self.translation = Matrix4::from_translation(motion * self.zoom_speed * elapsed) * self.translation;
        self.camera = self.translation * self.look_at * Matrix4::from(self.rotation);
        self.inv_camera = self.camera.invert().unwrap();
    }
    /// Pan the camera following the delta motion of the mouse. Mouse delta should be in pixels.
    pub fn pan(&mut self, mouse_delta: Vector2<f32>, elapsed: f32) {
        let motion = Vector3::new(mouse_delta.x, mouse_delta.y, 0.0) * self.motion_speed * elapsed;
        self.translation = Matrix4::from_translation(motion) * self.translation;
        self.camera = self.translation * self.look_at * Matrix4::from(self.rotation);
        self.inv_camera = self.camera.invert().unwrap();
    }
    /// Update the screen dimensions if the window has resized.
    pub fn update_screen(&mut self, width: f32, height: f32) {
        self.inv_screen[0] = 1.0 / width;
        self.inv_screen[1] = 1.0 / height;
    }
    fn screen_to_arcball(p: Vector2<f32>) -> Quaternion<f32> {
        let dist = cgmath::dot(p, p);
        // If we're on/in the sphere return the point on it
        if dist <= 1.0 {
            Quaternion::new(0.0, p.x, p.y, f32::sqrt(1.0 - dist))
        } else {
            let unit_p = p.normalize();
            Quaternion::new(0.0, unit_p.x, unit_p.y, 0.0)
        }
    }
}

fn clamp(x: f32, min: f32, max: f32) -> f32 {
    if x < min {
        min
    } else if x > max {
        max
    } else {
        x
    }
}

