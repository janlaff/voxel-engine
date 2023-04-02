#![feature(int_roundings)]

mod allocators;
mod camera;
mod command;
mod compute;
mod context;
mod gpu_model;
mod mouse;
mod swapchain;

use allocators::*;
use camera::*;
use command::*;
use compute::*;
use context::*;
use mouse::*;
use std::cell::RefCell;
use std::sync::Arc;
use swapchain::*;

use voxel_engine_shader::glam::{vec3, Vec3};
use voxel_engine_shader::{OctreeNode, Ray};
use vulkano::swapchain::{
    AcquireError, SwapchainCreateInfo, SwapchainCreationError, SwapchainPresentInfo,
};
use vulkano::sync;
use vulkano::sync::{FlushError, GpuFuture};
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, Event, MouseButton, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{CursorIcon, WindowBuilder};

fn main() {
    run_app();
}

fn run_app() {
    let event_loop = EventLoop::new();
    let window_builder = WindowBuilder::new().with_title("voxel-engine");

    let ctx = Context::new(&event_loop, window_builder);
    let allocators = Allocators::new(&ctx.gpu.device);

    let camera = RefCell::new(Camera::new(
        Vec3::splat(-3.0),
        Vec3::splat(0.0),
        ctx.window().inner_size().to_logical(1.0),
    ));

    let (mut swapchain, mut images) =
        create_swapchain(&ctx.gpu.device, &ctx.surface, ctx.window().inner_size());

    let octree = vec![
        // Root node
        OctreeNode::new(1, false, 0b00111111, 0b00000000),
        // First 8 sub cubes
        OctreeNode::new(8, false, 0b11111111, 0b11111110),
        OctreeNode::new(7, false, 0b11111111, 0b11111110),
        OctreeNode::new(6, false, 0b11111111, 0b11111110),
        OctreeNode::new(5, false, 0b11111111, 0b11111110),
        OctreeNode::new(4, false, 0b11111111, 0b11111110),
        OctreeNode::new(3, false, 0b11111111, 0b11111110),
        OctreeNode::new(2, false, 0b11111111, 0b11111110),
        OctreeNode::new(1, false, 0b11111111, 0b11111110),
        // 2nd level subcube
        OctreeNode::new(0, false, 0b11111111, 0b11111111),
        OctreeNode::new(0, false, 0b11111111, 0b11111111),
        OctreeNode::new(0, false, 0b11111111, 0b11111111),
        OctreeNode::new(0, false, 0b11111111, 0b11111111),
        OctreeNode::new(0, false, 0b11111111, 0b11111111),
        OctreeNode::new(0, false, 0b11111111, 0b11111111),
        OctreeNode::new(0, false, 0b11111111, 0b11111111),
        OctreeNode::new(0, false, 0b11111111, 0b11111111),
    ];

    let mut compute = Compute::new(
        &ctx.gpu.device,
        &ctx.gpu.queue,
        ctx.window().inner_size(),
        octree.clone(),
        &allocators,
    );

    {
        let mut writer = compute.camera_buffer.write().unwrap();
        *writer = camera.borrow().matrices();
    }

    let mut command_buffers = record_command_buffers(
        &ctx.gpu.device,
        &ctx.gpu.queue,
        &compute.pipeline,
        &images,
        &allocators.command_buffer,
        &compute.render_image_set,
        &compute.render_image,
    );

    let mut window_resized = false;
    let mut recreate_swapchain = false;

    let mut mouse_handler = MouseHandler::new();

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { event, .. } => {
            mouse_handler.process_event(&event, |drag_delta| {
                camera
                    .borrow_mut()
                    .arcball_rotate(drag_delta, ctx.window().inner_size().to_logical(1.0));

                let mut writer = compute.camera_buffer.write().unwrap();
                *writer = camera.borrow().matrices();
            });

            match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::Resized(_) => {
                    window_resized = true;
                }
                _ => {}
            }
        }
        Event::MainEventsCleared => {
            if window_resized || recreate_swapchain {
                recreate_swapchain = false;

                (swapchain, images) = match swapchain.recreate(SwapchainCreateInfo {
                    image_extent: ctx.window().inner_size().into(),
                    ..swapchain.create_info()
                }) {
                    Ok(r) => r,
                    Err(SwapchainCreationError::ImageExtentNotSupported { .. }) => return,
                    Err(e) => panic!("Failed to recreate swapchain: {}", e),
                };

                if window_resized {
                    window_resized = false;

                    compute = Compute::new(
                        &ctx.gpu.device,
                        &ctx.gpu.queue,
                        ctx.window().inner_size(),
                        octree.clone(),
                        &allocators,
                    );

                    camera
                        .borrow_mut()
                        .update_projection(ctx.window().inner_size().to_logical(1.0));

                    {
                        let mut writer = compute.camera_buffer.write().unwrap();
                        *writer = camera.borrow().matrices();
                    }
                }

                command_buffers = record_command_buffers(
                    &ctx.gpu.device,
                    &ctx.gpu.queue,
                    &compute.pipeline,
                    &images,
                    &allocators.command_buffer,
                    &compute.render_image_set,
                    &compute.render_image,
                );
            }

            let (image_index, suboptimal, acquire_future) =
                match vulkano::swapchain::acquire_next_image(swapchain.clone(), None) {
                    Ok(r) => r,
                    Err(AcquireError::OutOfDate) => {
                        recreate_swapchain = true;
                        return;
                    }
                    Err(e) => panic!("Failed to acquire next image: {:?}", e),
                };

            if suboptimal {
                recreate_swapchain = true;
            }

            let execution = sync::now(ctx.gpu.device.clone())
                .join(acquire_future)
                .then_execute(
                    ctx.gpu.queue.clone(),
                    command_buffers[image_index as usize].clone(),
                )
                .unwrap()
                .then_swapchain_present(
                    ctx.gpu.queue.clone(),
                    SwapchainPresentInfo::swapchain_image_index(swapchain.clone(), image_index),
                )
                .then_signal_fence_and_flush();

            match execution {
                Ok(future) => {
                    future.wait(None).unwrap();
                }
                Err(FlushError::OutOfDate) => {
                    recreate_swapchain = true;
                }
                Err(e) => {
                    println!("Failed to flush future: {:?}", e);
                }
            }
        }
        _ => {}
    });
}
