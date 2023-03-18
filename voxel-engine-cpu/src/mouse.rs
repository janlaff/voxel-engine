use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, MouseButton, WindowEvent};

pub struct MouseHandler {
    is_dragging: bool,
    last_position: PhysicalPosition<f64>
}

impl MouseHandler {
    pub fn new() -> Self {
        Self {
            is_dragging: false,
            last_position: PhysicalPosition::default(),
        }
    }

    pub fn process_event(&mut self, event: &WindowEvent, drag_handler: impl Fn(PhysicalPosition<f32>)) {
        match event {
            WindowEvent::MouseInput {
                ref state,
                button: MouseButton::Left,
                ..
            } => match state {
                ElementState::Pressed => {
                    self.is_dragging = true;
                }
                ElementState::Released => {
                    self.is_dragging = false;
                }
            },
            WindowEvent::CursorMoved { ref position, .. } => {
                if self.is_dragging {
                    let delta = PhysicalPosition::from((
                        position.x - self.last_position.x,
                        position.y - self.last_position.y,
                    ));

                    drag_handler(delta);
                }

                self.last_position = *position;
            }
            _ => {}
        }
    }
}