mod allocators;
mod camera;
mod compute;
mod context;

use allocators::*;
use camera::*;
use compute::*;
use context::*;

use std::default::Default;
use std::sync::Arc;

use voxel_engine_gpu::glam::{Vec2, Vec3};
use voxel_engine_gpu::OctreeNodeBuilder;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, BlitImageInfo, CommandBufferUsage, PrimaryAutoCommandBuffer,
};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAlloc;
use vulkano::descriptor_set::PersistentDescriptorSet;
use vulkano::device::{Device, Queue};
use vulkano::format::Format;
use vulkano::image::{ImageAccess, ImageUsage, StorageImage, SwapchainImage};
use vulkano::pipeline::{ComputePipeline, Pipeline, PipelineBindPoint};
use vulkano::swapchain::{
    PresentMode, Surface, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo,
};
use vulkano::sync::GpuFuture;
use vulkano::{swapchain, sync};
use winit::event::{DeviceEvent, ElementState, Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{CursorIcon, WindowBuilder};

fn main() {
    let event_loop = EventLoop::new();
    let window_builder = WindowBuilder::new()
        .with_title("rfox")
        .with_maximized(true)
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
                match swapchain::acquire_next_image(swapchain.clone(), None) {
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

fn record_command_buffers(
    device: &Arc<Device>,
    queue: &Arc<Queue>,
    pipeline: &Arc<ComputePipeline>,
    images: &Vec<Arc<SwapchainImage>>,
    command_buffer_allocator: &StandardCommandBufferAllocator,
    compute_image_set: &Arc<PersistentDescriptorSet<StandardDescriptorSetAlloc>>,
    compute_image: &Arc<StorageImage>,
) -> Vec<Arc<PrimaryAutoCommandBuffer>> {
    images
        .iter()
        .map(|swapchain_image| {
            let mut builder = AutoCommandBufferBuilder::primary(
                command_buffer_allocator,
                queue.queue_family_index(),
                CommandBufferUsage::MultipleSubmit,
            )
            .unwrap();

            builder
                .bind_pipeline_compute(pipeline.clone())
                .bind_descriptor_sets(
                    PipelineBindPoint::Compute,
                    pipeline.layout().clone(),
                    0,
                    compute_image_set.clone(),
                )
                .dispatch([
                    swapchain_image.dimensions().width() / 10,
                    swapchain_image.dimensions().height() / 10,
                    1,
                ])
                .unwrap()
                .blit_image(BlitImageInfo::images(
                    compute_image.clone(),
                    swapchain_image.clone(),
                ))
                .unwrap();

            Arc::new(builder.build().unwrap())
        })
        .collect()
}

fn create_swapchain(
    device: &Arc<Device>,
    surface: &Arc<Surface>,
    screen_size: (u32, u32),
) -> (Arc<Swapchain>, Vec<Arc<SwapchainImage>>) {
    let caps = device
        .physical_device()
        .surface_capabilities(surface, Default::default())
        .expect("Failed to get surface capabilities");

    let composite_alpha = caps.supported_composite_alpha.iter().next().unwrap();

    let format = *device
        .physical_device()
        .surface_formats(surface, Default::default())
        .unwrap()
        .iter()
        .find(|(f, c)| *f == Format::B8G8R8A8_SRGB)
        .unwrap();
    //.for_each(|(f, c)| println!("{:?} {:?}", *f, *c));

    Swapchain::new(
        device.clone(),
        surface.clone(),
        SwapchainCreateInfo {
            min_image_count: caps.min_image_count + 1,
            image_format: Some(format.0),
            image_color_space: format.1,
            image_extent: [screen_size.0, screen_size.1],
            image_usage: ImageUsage {
                transfer_dst: true,
                ..Default::default()
            },
            present_mode: PresentMode::Mailbox,
            clipped: false,
            composite_alpha,
            ..Default::default()
        },
    )
    .unwrap()
}
