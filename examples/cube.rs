#[macro_use]
extern crate glium;
extern crate arcball;
extern crate cgmath;

use glium::Surface;
use glium::index::PrimitiveType;
use glium::glutin::{self, ElementState, Event, VirtualKeyCode, MouseButton, MouseScrollDelta};
use cgmath::{Point3, Vector3, Vector2, Matrix4};
use arcball::ArcballCamera;

#[derive(Copy, Clone)]
struct Vertex {
    pos: [f32; 3],
    color: [f32; 3],
}
implement_vertex!(Vertex, pos, color);

fn main() {
    use glium::DisplayBuild;

    let display = glutin::WindowBuilder::new()
        .with_title("Arcball Camera Cube Example")
        .build_glium()
        .unwrap();

    // Hard-coded cube triangle strip
    let vertex_buffer = glium::VertexBuffer::new(&display,
        &[Vertex { pos: [1.0, 1.0, -1.0], color: [1.0, 0.0, 0.0] },
            Vertex { pos: [-1.0, 1.0, -1.0], color: [1.0, 0.0, 0.0] },
            Vertex { pos: [1.0, 1.0, 1.0], color: [1.0, 0.0, 0.0] },

            Vertex { pos: [-1.0, 1.0, 1.0], color: [0.0, 1.0, 0.0] },
            Vertex { pos: [-1.0, -1.0, 1.0], color: [0.0, 1.0, 0.0] },
            Vertex { pos: [-1.0, 1.0, -1.0], color: [0.0, 1.0, 0.0] },

            Vertex { pos: [-1.0, -1.0, -1.0], color: [0.0, 0.0, 1.0] },
            Vertex { pos: [1.0, 1.0, -1.0], color: [0.0, 0.0, 1.0] },
            Vertex { pos: [1.0, -1.0, -1.0], color: [0.0, 0.0, 1.0] },

            Vertex { pos: [1.0, 1.0, 1.0], color: [1.0, 1.0, 0.0] },
            Vertex { pos: [1.0, -1.0, 1.0], color: [1.0, 1.0, 0.0] },
            Vertex { pos: [-1.0, -1.0, 1.0], color: [1.0, 1.0, 0.0] },

            Vertex { pos: [1.0, -1.0, -1.0], color: [1.0, 0.0, 1.0] },
            Vertex { pos: [-1.0, -1.0, -1.0], color: [1.0, 0.0, 1.0] }
        ]
    ).unwrap();
    let index_buffer = glium::index::NoIndices(PrimitiveType::TriangleStrip);

    let program = program!(&display,
        140 => {
            vertex: "
                #version 140

                uniform mat4 proj_view;

                in vec3 pos;
                in vec3 color;

                out vec3 vcolor;

                void main(void) {
                    gl_Position = proj_view * vec4(pos, 1.0);
                    vcolor = color;
                }
            ",
            fragment: "
                #version 140

                in vec3 vcolor;
                out vec4 color;

                void main(void) {
                    color = vec4(vcolor, 1.0);
                }
            "
        },
    ).unwrap();

    let display_dims = display.get_framebuffer_dimensions();
    let mut persp_proj = cgmath::perspective(cgmath::Deg(65.0), display_dims.0 as f32 / display_dims.1 as f32, 1.0, 200.0);
    let mut arcball_camera = {
        let look_at = Matrix4::<f32>::look_at(Point3::new(0.0, 0.0, 6.0),
                                              Point3::new(0.0, 0.0, 0.0),
                                              Vector3::new(0.0, 1.0, 0.0));
        ArcballCamera::new(&look_at, 0.05, 4.0, [display_dims.0 as f32, display_dims.1 as f32])
    };

    // Track if left/right mouse is down
    let mut mouse_pressed = [false, false];
    let mut prev_mouse = None;
    'outer: loop {
        for e in display.poll_events() {
            match e {
                glutin::Event::Closed => break 'outer,
                Event::KeyboardInput(state, _, code) => {
                    let pressed = state == ElementState::Pressed;
                    match code {
                        Some(VirtualKeyCode::Escape) if pressed => break 'outer,
                        _ => {}
                    }
                },
                Event::MouseMoved(x, y) if prev_mouse.is_none() => {
                    prev_mouse = Some((x, y));
                },
                Event::MouseMoved(x, y) => {
                    let prev = prev_mouse.unwrap();
                    if mouse_pressed[0] {
                        arcball_camera.rotate(Vector2::new(prev.0 as f32, prev.1 as f32), Vector2::new(x as f32, y as f32));
                    } else if mouse_pressed[1] {
                        let mouse_delta = Vector2::new((x - prev.0) as f32, -(y - prev.1) as f32);
                        arcball_camera.pan(mouse_delta, 0.16);
                    }
                    prev_mouse = Some((x, y));
                },
                Event::MouseInput(state, button) => {
                    if button == MouseButton::Left {
                        mouse_pressed[0] = state == ElementState::Pressed;
                    } else if button == MouseButton::Right {
                        mouse_pressed[1] = state == ElementState::Pressed;
                    }
                },
                Event::MouseWheel(delta, _) => {
                    let y = match delta {
                        MouseScrollDelta::LineDelta(_, y) => y,
                        MouseScrollDelta::PixelDelta(_, y) => y,
                    };
                    arcball_camera.zoom(y, 0.16);
                },
                Event::Resized(w, h) => {
                    persp_proj = cgmath::perspective(cgmath::Deg(65.0), w as f32 / h as f32, 1.0, 1000.0);
                    arcball_camera.update_screen(w as f32, h as f32);
                },
                _ => {}
            }
        }
        let proj_view: [[f32; 4]; 4] = (persp_proj * arcball_camera.get_mat4()).into();
        let uniforms = uniform! {
            proj_view: proj_view,
        };
        let draw_params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::draw_parameters::DepthTest::IfLess,
                write: true,
                .. Default::default()
            },
            .. Default::default()
        };

        let mut target = display.draw();
        target.clear_color(0.1, 0.1, 0.1, 0.0);
        target.clear_depth(1.0);
        target.draw(&vertex_buffer, &index_buffer, &program, &uniforms, &draw_params).unwrap();
        target.finish().unwrap();
    }
}

