mod allocators;
mod camera;
mod command;
mod compute;
mod context;
mod gpu_model;
mod swapchain;

use allocators::*;
use camera::*;
use command::*;
use compute::*;
use context::*;
use swapchain::*;

use voxel_engine_gpu::glam::{Vec2, Vec3};
use voxel_engine_gpu::OctreeNodeBuilder;
use vulkano::swapchain::{
    AcquireError, SwapchainCreateInfo, SwapchainCreationError, SwapchainPresentInfo,
};
use vulkano::sync;
use vulkano::sync::{FlushError, GpuFuture};
use winit::dpi::PhysicalPosition;
use winit::event::{DeviceEvent, ElementState, Event, MouseButton, WindowEvent};
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

    let mut camera = Camera::new(
        Vec3::splat(3.0),
        Vec3::splat(0.0),
        ctx.window().inner_size().to_logical(1.0),
    );

    let (mut swapchain, mut images) =
        create_swapchain(&ctx.gpu.device, &ctx.surface, ctx.window().inner_size());

    let octree = vec![
        OctreeNodeBuilder::new()
            .valid(0b10101010)
            .leaf(0b00100000)
            .build(),
        OctreeNodeBuilder::new()
            .valid(0b11111111)
            .leaf(0b11111111)
            .build(),
        OctreeNodeBuilder::new()
            .valid(0b00000000)
            .leaf(0b00000000)
            .build(),
        OctreeNodeBuilder::new()
            .valid(0b00000000)
            .leaf(0b00000000)
            .build(),
        OctreeNodeBuilder::new()
            .valid(0b00000000)
            .leaf(0b00000000)
            .build(),
        OctreeNodeBuilder::new()
            .valid(0b00000000)
            .leaf(0b00000000)
            .build(),
        OctreeNodeBuilder::new()
            .valid(0b00000000)
            .leaf(0b00000000)
            .build(),
        OctreeNodeBuilder::new()
            .valid(0b00000000)
            .leaf(0b00000000)
            .build(),
        OctreeNodeBuilder::new()
            .valid(0b00000000)
            .leaf(0b00000000)
            .build(),
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
        *writer = camera.inverse();
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

    let mut dragging = false;
    let mut window_resized = false;
    let mut recreate_swapchain = false;

    let mut last_position = PhysicalPosition::default();

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            *control_flow = ControlFlow::Exit;
        }
        Event::WindowEvent {
            event: WindowEvent::Resized(_),
            ..
        } => {
            window_resized = true;
        }
        Event::WindowEvent {
            event: WindowEvent::CursorMoved { position, .. },
            ..
        } => {
            if dragging {
                let delta = PhysicalPosition::from((
                    position.x - last_position.x,
                    position.y - last_position.y,
                ));

                camera.arcball_rotate(
                    delta,
                    ctx.window().inner_size().to_logical(1.0),
                );

                let mut writer = compute.camera_buffer.write().unwrap();
                *writer = camera.inverse();
            }

            last_position = position;
        }
        Event::WindowEvent {
            event: WindowEvent::MouseInput { state, button: MouseButton::Left, .. },
            ..
        } => match state {
            ElementState::Pressed => {
                dragging = true;
                ctx.window().set_cursor_icon(CursorIcon::Move);
            }
            ElementState::Released => {
                dragging = false;
                ctx.window().set_cursor_icon(CursorIcon::Default);
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

                    camera.update_projection(ctx.window().inner_size().to_logical(1.0));

                    {
                        let mut writer = compute.camera_buffer.write().unwrap();
                        *writer = camera.inverse();
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
