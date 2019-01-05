#[macro_use]
extern crate glium;
extern crate arcball;
extern crate cgmath;

use glium::Surface;
use glium::index::PrimitiveType;
use glium::glutin::{self, ControlFlow, ElementState, Event, MouseButton, MouseScrollDelta,
                    VirtualKeyCode, WindowEvent};
use cgmath::{Point3, Vector3, Vector2, Matrix4};
use arcball::ArcballCamera;

#[derive(Copy, Clone)]
struct Vertex {
    pos: [f32; 3],
    color: [f32; 3],
}
implement_vertex!(Vertex, pos, color);

fn main() {

    let window = glutin::WindowBuilder::new()
        .with_title("Arcball Camera Cube Example");
    let context = glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_depth_buffer(24);
    let mut events_loop = glutin::EventsLoop::new();

    let display =
        glium::Display::new(window, context, &events_loop).expect("failed to create display");

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
    let mut persp_proj = cgmath::perspective(cgmath::Deg(65.0), display_dims.0 as f32 / display_dims.1 as f32,
                                             1.0, 200.0);
    let mut arcball_camera = {
        let look_at = Matrix4::<f32>::look_at(Point3::new(0.0, 0.0, 6.0),
                                              Point3::new(0.0, 0.0, 0.0),
                                              Vector3::new(0.0, 1.0, 0.0));
        ArcballCamera::new(&look_at, 0.05, 4.0, [display_dims.0 as f32, display_dims.1 as f32])
    };

    // Track if left/right mouse is down
    let mut mouse_pressed = [false, false];
    let mut prev_mouse: Option<(f32, f32)> = None;

    // TODO: Seems like there's some odd delay or buffering in the event
    // loop that gives this weird latency feel to the interaction and rendering.
    // Should fix this
    events_loop.run_forever(|e| {
        match e {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => return ControlFlow::Break,
                WindowEvent::KeyboardInput { input, .. } => {
                    let code = input.virtual_keycode;
                    let pressed = input.state == ElementState::Pressed;
                    match code {
                        Some(VirtualKeyCode::Escape) if pressed => return ControlFlow::Break,
                        _ => {}
                    }
                }

                WindowEvent::CursorMoved { position, .. } => {
                    let x = position.x as f32;
                    let y = position.y as f32;

                    if prev_mouse.is_none() {
                        prev_mouse = Some((x, y));
                    } else {
                        let prev = prev_mouse.unwrap();
                        if mouse_pressed[0] {
                            arcball_camera.rotate(
                                Vector2::new(prev.0, prev.1),
                                Vector2::new(x, y),
                            );
                        } else if mouse_pressed[1] {
                            let mouse_delta =
                                Vector2::new(x - prev.0, -(y - prev.1));
                            arcball_camera.pan(mouse_delta, 0.16);
                        }
                        prev_mouse = Some((x, y));
                    }
                }
                WindowEvent::MouseInput { state, button, .. } => {
                    if button == MouseButton::Left {
                        mouse_pressed[0] = state == ElementState::Pressed;
                    } else if button == MouseButton::Right {
                        mouse_pressed[1] = state == ElementState::Pressed;
                    }
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    let y = match delta {
                        MouseScrollDelta::LineDelta(_, y) => y,
                        MouseScrollDelta::PixelDelta(p) => p.y as f32,
                    };
                    arcball_camera.zoom(y, 0.16);
                }
                WindowEvent::Resized(size) => {
                    let w = size.width as f32;
                    let h = size.height as f32;
                    persp_proj =
                        cgmath::perspective(cgmath::Deg(65.0), w / h, 1.0, 1000.0);
                    arcball_camera.update_screen(w, h);
                }
                _ => {
                    return ControlFlow::Continue;
                }
            },
            _ => {
                return ControlFlow::Continue;
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
                ..Default::default()
            },
            ..Default::default()
        };

        let mut target = display.draw();
        target.clear_color(0.1, 0.1, 0.1, 0.0);
        target.clear_depth(1.0);
        target.draw(&vertex_buffer, &index_buffer, &program, &uniforms, &draw_params).unwrap();
        target.finish().unwrap();

        return ControlFlow::Continue;
    });
}


