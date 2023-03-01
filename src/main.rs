use std::ffi::CString;

use gl::types::*;

use glfw::Context;

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    glfw.window_hint(glfw::WindowHint::Resizable(false));
    glfw.window_hint(glfw::WindowHint::ContextVersion(4, 6));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));

    let (mut window, events) = glfw
        .create_window(800, 600, "framb", glfw::WindowMode::Windowed)
        .unwrap();

    let (width, height) = window.get_size();
    println!("{:?}", (width, height));

    window.make_current();
    glfw.set_swap_interval(glfw::SwapInterval::None);

    // Load OpenGL function pointers
    gl::load_with(|s| window.get_proc_address(s) as *const _);

    unsafe {
        // OpenGL initialization
        gl::ClearColor(1.0, 1.0, 1.0, 1.0);
        gl::Viewport(0, 0, width, height);

        let shader = gl::CreateShader(gl::COMPUTE_SHADER);
        gl::ShaderSource(
            shader,
            1,
            &CString::new(include_str!("red.comp")).unwrap().as_ptr(),
            std::ptr::null(),
        );
        gl::CompileShader(shader);

        let mut result: GLint = 0;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut result);

        if result != 1 {
            let mut log_len: GLint = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut log_len);

            let mut buffer = Vec::with_capacity(log_len as usize + 1);
            buffer.extend([b' '].iter().cycle().take(log_len as usize));

            let error: CString = CString::from_vec_unchecked(buffer);
            gl::GetShaderInfoLog(
                shader,
                log_len,
                std::ptr::null_mut(),
                error.as_ptr() as *mut GLchar,
            );

            let error = error.to_string_lossy().into_owned();
            println!("{}", error);
        }

        println!("{}", result);

        let program = gl::CreateProgram();
        gl::AttachShader(program, shader);
        gl::LinkProgram(program);
        gl::UseProgram(program);

        let mut output_texture: GLuint = 0;
        gl::GenTextures(1, &mut output_texture);
        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, output_texture);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA32F as GLint,
            width,
            height,
            0,
            gl::RGBA,
            gl::FLOAT,
            std::ptr::null(),
        );
        gl::BindImageTexture(
            0,
            output_texture,
            0,
            gl::FALSE,
            0,
            gl::WRITE_ONLY,
            gl::RGBA32F,
        );

        let mut framebuffer: GLuint = 0;
        gl::GenFramebuffers(1, &mut framebuffer);
        gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffer);
        gl::FramebufferTexture2D(
            gl::FRAMEBUFFER,
            gl::COLOR_ATTACHMENT0,
            gl::TEXTURE_2D,
            output_texture,
            0,
        );
        gl::BindFramebuffer(gl::READ_FRAMEBUFFER, framebuffer);
        gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, 0);

        let mut last_frame = glfw.get_time();
        let mut current_frame = last_frame;
        let mut fps = 0;

        while !window.should_close() {
            current_frame = glfw.get_time();

            if current_frame - last_frame > 1.0 {
                println!("FPS: {}", fps);

                last_frame = current_frame;
                fps = 0;
            }
            fps += 1;

            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::DispatchCompute(width as GLuint / 10, height as GLuint / 10, 1);
            gl::MemoryBarrier(gl::SHADER_IMAGE_ACCESS_BARRIER_BIT);
            gl::BlitFramebuffer(
                0,
                0,
                width,
                height,
                0,
                0,
                width,
                height,
                gl::COLOR_BUFFER_BIT,
                gl::LINEAR,
            );

            window.swap_buffers();
            glfw.poll_events();

            for (_, event) in glfw::flush_messages(&events) {
                match event {
                    glfw::WindowEvent::FramebufferSize(width, height) => {
                        //gl::Viewport(0, 0, width, height);
                    }
                    _ => {}
                }
            }
        }
    }
}
