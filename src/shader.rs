use gl::types::*;
use std::ffi::CString;

pub struct Shader(pub GLuint);

impl Shader {
    pub unsafe fn from_glsl(kind: GLenum, glsl_src_code: &str) -> Result<Self, String> {
        let shader = gl::CreateShader(kind);

        // Load source code
        let temp = CString::new(glsl_src_code).unwrap();
        gl::ShaderSource(shader, 1, &temp.as_ptr(), std::ptr::null());

        gl::CompileShader(shader);

        // Check that compilation was succesful
        check_compile_status(shader)?;

        Ok(Self(shader))
    }

    #[allow(dead_code, unused_variables)]
    pub unsafe fn from_spirv(
        kind: GLenum,
        spirv_bytes: &[u8],
        entry_point: &str,
    ) -> Result<Self, String> {
        unimplemented!()
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
