use gl::types::*;
use std::ffi::{c_void, CString};

pub struct Shader(pub GLuint);

impl Shader {
    pub fn from_glsl(kind: GLenum, glsl_src_code: &str) -> Result<Self, String> {
        unsafe {
            let shader = gl::CreateShader(kind);

            // Load source code
            let temp = CString::new(glsl_src_code).unwrap();
            gl::ShaderSource(shader, 1, &temp.as_ptr(), std::ptr::null());

            gl::CompileShader(shader);

            // Check that compilation was succesful
            check_compile_status(shader)?;

            Ok(Self(shader))
        }
    }

    pub fn from_spirv(kind: GLenum, spirv_bytes: &[u8], entry_point: &str) -> Result<Self, String> {
        unsafe {
            let shader = gl::CreateShader(kind);

            // Load SPIR-V binary
            gl::ShaderBinary(
                1,
                &shader,
                gl::SHADER_BINARY_FORMAT_SPIR_V,
                spirv_bytes.as_ptr() as *mut c_void,
                spirv_bytes.len() as i32,
            );

            // Specialization is equal to compilation
            let temp = CString::new(entry_point).unwrap();
            gl::SpecializeShader(shader, temp.as_ptr(), 0, std::ptr::null(), std::ptr::null());

            // Check that specialization was successful
            check_compile_status(shader)?;

            Ok(Self(shader))
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.0);
        }
    }
}

unsafe fn check_compile_status(shader: GLuint) -> Result<(), String> {
    let mut compile_status: GLint = 0;
    gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut compile_status);

    if compile_status == 0 {
        let mut log_len: GLint = 0;
        gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut log_len);

        // Create buffer for opengl to store the error
        let error = CString::from_vec_unchecked(vec![0x00; log_len as usize]);

        gl::GetShaderInfoLog(
            shader,
            log_len,
            std::ptr::null_mut(),
            error.as_ptr() as *mut GLchar,
        );

        // Convert string and return as error
        return Err(error.to_string_lossy().into_owned());
    }

    Ok(())
}
