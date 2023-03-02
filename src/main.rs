use std::ffi::CString;

use gl::types::*;
use glfw::Context;

mod program;
mod shader;

use program::*;
use shader::*;

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    glfw.window_hint(glfw::WindowHint::ContextVersion(4, 6));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));

    let (mut window, events) = glfw
        .create_window(800, 600, "framb", glfw::WindowMode::Windowed)
        .unwrap();

    let (mut width, mut height) = window.get_framebuffer_size();

    window.make_current();
    glfw.set_swap_interval(glfw::SwapInterval::None);

    gl::load_with(|s| window.get_proc_address(s) as *const _);

    unsafe {
        let program =
            Program::new(&[
                Shader::from_glsl(gl::COMPUTE_SHADER, include_str!("red.comp")).unwrap(),
            ])
            .unwrap();

        program.set_used();

        let group_size = program.work_group_size();

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
        let mut fps = 0;

        window.set_framebuffer_size_polling(true);
        while !window.should_close() {
            let current_frame = glfw.get_time();
            if current_frame - last_frame > 1.0 {
                println!("FPS: {}", fps);

                last_frame = current_frame;
                fps = 0;
            }
            fps += 1;

            gl::DispatchCompute(
                ((width + group_size[0] - 1) / group_size[0]) as GLuint,
                ((height + group_size[1] - 1) / group_size[1]) as GLuint,
                1,
            );
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
                    glfw::WindowEvent::FramebufferSize(new_width, new_height) => {
                        width = new_width;
                        height = new_height;

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
                    }
                    _ => {}
                }
            }
        }
    }
}
