use std::ffi::c_void;

use gl::types::*;

pub struct UniformBuffer {
    buffer: GLuint,
}

impl UniformBuffer {
    pub fn new(binding_index: GLuint) -> Self {
        let mut buffer: GLuint = 0;

        unsafe {
            gl::GenBuffers(1, &mut buffer);
            gl::BindBuffer(gl::UNIFORM_BUFFER, buffer);
            gl::BindBufferBase(gl::UNIFORM_BUFFER, binding_index, buffer);
        }

        Self { buffer }
    }

    pub fn write(&mut self, data: &[u8]) {
        unsafe {
            gl::BufferData(
                gl::UNIFORM_BUFFER,
                data.len() as isize,
                data.as_ptr() as *const c_void,
                gl::DYNAMIC_DRAW,
            );
        }
    }
}

impl Drop for UniformBuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.buffer);
        }
    }
}
