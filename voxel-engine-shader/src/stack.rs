use glam::Vec3;

const STACK_SIZE: usize = 8;

pub struct Stack {
    index: [usize; STACK_SIZE],
    center: [Vec3; STACK_SIZE],
    scale: [f32; STACK_SIZE],
    ptr: i32,
}

impl Stack {
    pub fn new() -> Self {
        Self {
            index: [0; STACK_SIZE],
            center: [Vec3::default(); STACK_SIZE],
            scale: [0.0; STACK_SIZE],
            ptr: -1,
        }
    }

    pub fn empty(&self) -> bool {
        self.ptr < 0
    }

    pub fn push(&mut self, index: usize, center: Vec3, scale: f32) {
        self.ptr += 1;
        self.index[self.ptr as usize] = index;
        self.center[self.ptr as usize] = center;
        self.scale[self.ptr as usize] = scale;
    }

    pub fn pop(&mut self) -> (usize, Vec3, f32) {
        self.ptr -= 1;
        (
            self.index[(self.ptr + 1) as usize],
            self.center[(self.ptr + 1) as usize],
            self.scale[(self.ptr + 1) as usize],
        )
    }
}

#[test]
fn test_stack() {
    let mut stack = Stack::new();

    assert!(stack.empty());
    stack.push(0, Vec3::default(), 0.0);
    assert!(!stack.empty());
    assert_eq!(stack.pop(), (0, Vec3::default(), 0.0));
    assert!(stack.empty());

    stack.push(0, Vec3::default(), 0.0);
    stack.push(1, Vec3::default(), 0.1);

    assert_eq!(stack.pop(), (1, Vec3::default(), 0.1));
    assert_eq!(stack.pop(), (0, Vec3::default(), 0.0));
}
