use crate::shader::Shader;
use gl::types::*;
use std::ffi::CString;

pub struct Program(pub GLuint);

impl Program {
    pub unsafe fn new(shaders: &[Shader]) -> Result<Self, String> {
        let program = gl::CreateProgram();

        // Attach shaders to the program
        shaders.iter().for_each(|s| gl::AttachShader(program, s.0));

        gl::LinkProgram(program);

        // Detach shaders from the program
        shaders.iter().for_each(|s| gl::DetachShader(program, s.0));

        // Check that linking was succesful
        check_link_status(program)?;

        Ok(Self(program))
    }

    pub unsafe fn set_used(&self) {
        gl::UseProgram(self.0);
    }

    pub unsafe fn work_group_size(&self) -> [GLint; 3] {
        let mut group_size: [GLint; 3] = [0; 3];
        gl::GetProgramiv(
            self.0,
            gl::COMPUTE_WORK_GROUP_SIZE,
            group_size.as_mut_ptr(),
        );

        group_size
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.0);
        }
    }
}

unsafe fn check_link_status(program: GLuint) -> Result<(), String> {
    let mut link_status: GLint = 0;
    gl::GetProgramiv(program, gl::LINK_STATUS, &mut link_status);

    if link_status == 0 {
        let mut log_len: GLint = 0;
        gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut log_len);

        // Create buffer for opengl to store the error
        let error = CString::from_vec_unchecked(vec![0x00; log_len as usize]);

        gl::GetProgramInfoLog(
            program,
            log_len,
            std::ptr::null_mut(),
            error.as_ptr() as *mut GLchar,
        );

        // Convert string and return as error
        return Err(error.to_string_lossy().into_owned());
    }

    Ok(())
}
