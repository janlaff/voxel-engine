mod allocators;
mod camera;
mod command;
mod compute;
mod context;
mod device_selection;
mod swapchain;

use allocators::*;
use camera::*;
use command::*;
use compute::*;
use context::*;
use device_selection::*;
use swapchain::*;

use voxel_engine_gpu::glam::{Vec2, Vec3};
use voxel_engine_gpu::OctreeNodeBuilder;
use vulkano::swapchain::SwapchainPresentInfo;
use vulkano::sync;
use vulkano::sync::GpuFuture;
use winit::event::{DeviceEvent, ElementState, Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{CursorIcon, WindowBuilder};

fn main() {
    run_app();
}

fn run_app() {
    let event_loop = EventLoop::new();
    let window_builder = WindowBuilder::new()
        .with_title("voxel-engine")
        .with_resizable(false);

    let ctx = Context::new(&event_loop, window_builder);
    let allocators = Allocators::new(&ctx.device);

    let screen_size_u = (
        ctx.window().inner_size().width,
        ctx.window().inner_size().height,
    );

    let screen_size_f = (screen_size_u.0 as f32, screen_size_u.1 as f32);

    let mut camera = Camera::new(Vec3::splat(3.0), Vec3::splat(0.0), screen_size_f);

    let (swapchain, images) = create_swapchain(&ctx.device, &ctx.surface, screen_size_u);

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

    let compute = Compute::new(&ctx.device, &ctx.queue, screen_size_u, octree, &allocators);

    {
        let mut writer = compute.camera_buffer.write().unwrap();
        *writer = camera.inverse();
    }

    let command_buffers = record_command_buffers(
        &ctx.device,
        &ctx.queue,
        &compute.pipeline,
        &images,
        &allocators.command_buffer,
        &compute.render_image_set,
        &compute.render_image,
    );

    let mut dragging = false;
    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            *control_flow = ControlFlow::Exit;
        }
        Event::DeviceEvent { event, .. } => match event {
            DeviceEvent::MouseMotion { delta } => {
                if dragging {
                    camera.arcball_rotate(Vec2::new(delta.0 as f32, delta.1 as f32), screen_size_f);

                    let mut writer = compute.camera_buffer.write().unwrap();
                    *writer = camera.inverse();
                }
            }
            DeviceEvent::Button { button: 1, state } => match state {
                ElementState::Pressed => {
                    dragging = true;
                    ctx.window().set_cursor_icon(CursorIcon::Move);
                }
                ElementState::Released => {
                    dragging = false;
                    ctx.window().set_cursor_icon(CursorIcon::Default);
                }
            },
            _ => {}
        },
        Event::MainEventsCleared => {
            let (image_index, suboptimal, acquire_future) =
                match vulkano::swapchain::acquire_next_image(swapchain.clone(), None) {
                    Ok(r) => r,
                    Err(e) => panic!("Failed to acquire next image: {:?}", e),
                };

            let execution = sync::now(ctx.device.clone())
                .join(acquire_future)
                .then_execute(
                    ctx.queue.clone(),
                    command_buffers[image_index as usize].clone(),
                )
                .unwrap()
                .then_swapchain_present(
                    ctx.queue.clone(),
                    SwapchainPresentInfo::swapchain_image_index(swapchain.clone(), image_index),
                )
                .then_signal_fence_and_flush();

            match execution {
                Ok(future) => {
                    future.wait(None).unwrap();
                }
                Err(e) => {
                    println!("Failed to flush future: {:?}", e);
                }
            }
        }
        _ => {}
    });
}
