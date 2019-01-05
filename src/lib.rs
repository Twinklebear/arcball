//! An implementation of the [Shoemake Arcball Camera](https://www.talisman.org/~erlkonig/misc/shoemake92-arcball.pdf)
//! using [cgmath](https://crates.io/crates/cgmath). See the
//! [cube example](https://github.com/Twinklebear/arcball/blob/master/examples/cube.rs) for an example
//! of use with [glium](https://crates.io/crates/glium).
extern crate cgmath;

use cgmath::prelude::*;
use cgmath::{BaseFloat, Matrix4, Quaternion, Vector2, Vector3};
use cgmath::num_traits::clamp;

/// The Shoemake Arcball camera.
pub struct ArcballCamera<F> {
    look_at: Matrix4<F>,
    translation: Matrix4<F>,
    rotation: Quaternion<F>,
    camera: Matrix4<F>,
    inv_camera: Matrix4<F>,
    motion_speed: F,
    zoom_speed: F,
    inv_screen: [F; 2],
}

impl<F: BaseFloat> ArcballCamera<F> {
    /// Create a new Arcball camera starting from the look at matrix `look_at`. The `motion_speed`
    /// sets the speed for panning and `zoom_speed` the speed for zooming the camera. `screen` should be
    /// `[screen_width, screen_height]`.
    pub fn new(look_at: &Matrix4<F>, motion_speed: F, zoom_speed: F, screen: [F; 2]) -> ArcballCamera<F> {
        ArcballCamera {
            look_at: *look_at,
            translation: Transform::one(),
            rotation: Quaternion::new(F::one(), F::zero(), F::zero(), F::zero()),
            camera: *look_at,
            inv_camera: look_at.invert().unwrap(),
            motion_speed,
            zoom_speed,
            inv_screen: [F::one() / screen[0], F::one() / screen[1]],
        }
    }
    /// Get the view matrix computed by the camera.
    pub fn get_mat4(&self) -> Matrix4<F> {
        self.camera
    }
    /// Rotate the camera, mouse positions should be in pixel coordinates.
    ///
    /// Rotates from the orientation at the previous mouse position specified by `mouse_prev`
    /// to the orientation at the current mouse position, `mouse_cur`.
    pub fn rotate(&mut self, mouse_prev: Vector2<F>, mouse_cur: Vector2<F>) {
        let one = F::one();
        let two = F::from(2.0).unwrap();
        let m_cur = Vector2::new(clamp(mouse_cur.x * two * self.inv_screen[0] - one, -one, one),
                                    clamp(one - two * mouse_cur.y * self.inv_screen[1], -one, one));
        let m_prev = Vector2::new(clamp(mouse_prev.x * two * self.inv_screen[0] - one, -one, one),
                                    clamp(one - two * mouse_prev.y * self.inv_screen[1], -one, one));
        let mouse_cur_ball = ArcballCamera::screen_to_arcball(m_cur);
        let mouse_prev_ball = ArcballCamera::screen_to_arcball(m_prev);
        self.rotation = mouse_cur_ball * mouse_prev_ball * self.rotation;
        self.camera = self.translation * self.look_at * Matrix4::from(self.rotation);
        self.inv_camera = self.camera.invert().unwrap();
    }
    /// Zoom the camera in by some amount. Positive values zoom in, negative zoom out.
    pub fn zoom(&mut self, amount: F, elapsed: F) {
        let motion = Vector3::new(F::zero(), F::zero(), amount);
        self.translation = Matrix4::from_translation(motion * self.zoom_speed * elapsed) * self.translation;
        self.camera = self.translation * self.look_at * Matrix4::from(self.rotation);
        self.inv_camera = self.camera.invert().unwrap();
    }
    /// Pan the camera following the motion of the mouse. The mouse delta should be in pixels.
    pub fn pan(&mut self, mouse_delta: Vector2<F>, elapsed: F) {
        let motion = mouse_delta.extend(F::zero()) * self.motion_speed * elapsed;
        self.translation = Matrix4::from_translation(motion) * self.translation;
        self.camera = self.translation * self.look_at * Matrix4::from(self.rotation);
        self.inv_camera = self.camera.invert().unwrap();
    }
    /// Update the screen dimensions, e.g. if the window has resized.
    pub fn update_screen(&mut self, width: F, height: F) {
        self.inv_screen[0] = F::one() / width;
        self.inv_screen[1] = F::one() / height;
    }
    fn screen_to_arcball(p: Vector2<F>) -> Quaternion<F> {
        let dist = cgmath::dot(p, p);
        // If we're on/in the sphere return the point on it
        if dist <= F::one() {
            Quaternion::new(F::zero(), p.x, p.y, F::sqrt(F::one() - dist))
        } else {
            let unit_p = p.normalize();
            Quaternion::new(F::zero(), unit_p.x, unit_p.y, F::zero())
        }
    }
}
