use crate::allocators::Allocators;
use std::sync::Arc;
use voxel_engine_gpu::{InverseCamera, OctreeNode};
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAlloc;
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::device::{Device, Queue};
use vulkano::format::Format;
use vulkano::image::view::{ImageView, ImageViewCreateInfo};
use vulkano::image::{
    ImageAccess, ImageAspects, ImageDimensions, ImageSubresourceRange, ImageUsage, StorageImage,
};
use vulkano::pipeline::{ComputePipeline, Pipeline};
use vulkano::shader::ShaderModule;
use winit::dpi::PhysicalSize;

const RFOX_SHADER_BYTES: &[u8] = include_bytes!(env!("voxel_engine_gpu.spv"));

type OctreeBuffer = CpuAccessibleBuffer<[OctreeNode]>;
type CameraBuffer = CpuAccessibleBuffer<InverseCamera>;

pub struct Compute {
    pub pipeline: Arc<ComputePipeline>,
    pub camera_buffer: Arc<CameraBuffer>,
    pub octree_buffer: Arc<OctreeBuffer>,
    pub render_image: Arc<StorageImage>,
    pub render_image_view: Arc<ImageView<StorageImage>>,
    pub render_image_set: Arc<PersistentDescriptorSet<StandardDescriptorSetAlloc>>,
}

impl Compute {
    pub fn new(
        device: &Arc<Device>,
        queue: &Arc<Queue>,
        screen_size: PhysicalSize<u32>,
        octree: Vec<OctreeNode>,
        allocators: &Allocators,
    ) -> Self {
        let shader = create_shader(device);
        let pipeline = create_pipeline(device, shader);
        let camera_buffer = create_camera_buffer(allocators);
        let octree_buffer = create_octree_buffer(octree, allocators);
        let render_image = create_render_image(queue, screen_size, allocators);
        let render_image_view = create_render_image_view(&render_image);
        let render_image_set = create_render_image_set(
            &pipeline,
            &render_image_view,
            &camera_buffer,
            &octree_buffer,
            allocators,
        );

        Self {
            pipeline,
            camera_buffer,
            octree_buffer,
            render_image,
            render_image_view,
            render_image_set,
        }
    }
}

fn create_shader(device: &Arc<Device>) -> Arc<ShaderModule> {
    unsafe { ShaderModule::from_bytes(device.clone(), RFOX_SHADER_BYTES) }.unwrap()
}

fn create_pipeline(device: &Arc<Device>, shader: Arc<ShaderModule>) -> Arc<ComputePipeline> {
    ComputePipeline::new(
        device.clone(),
        shader.entry_point("main_cs").unwrap(),
        &(),
        None,
        |_| {},
    )
    .unwrap()
}

fn create_camera_buffer(allocators: &Allocators) -> Arc<CameraBuffer> {
    CpuAccessibleBuffer::from_data(
        &allocators.memory,
        BufferUsage {
            uniform_buffer: true,
            ..Default::default()
        },
        false,
        InverseCamera::default(),
    )
    .unwrap()
}

fn create_octree_buffer(octree: Vec<OctreeNode>, allocators: &Allocators) -> Arc<OctreeBuffer> {
    CpuAccessibleBuffer::from_iter(
        &allocators.memory,
        BufferUsage {
            storage_buffer: true,
            ..Default::default()
        },
        false,
        octree.into_iter(),
    )
    .unwrap()
}

fn create_render_image(
    queue: &Arc<Queue>,
    screen_size: PhysicalSize<u32>,
    allocators: &Allocators,
) -> Arc<StorageImage> {
    StorageImage::new(
        &allocators.memory,
        ImageDimensions::Dim2d {
            width: screen_size.width,
            height: screen_size.height,
            array_layers: 1,
        },
        Format::R32G32B32A32_SFLOAT,
        Some(queue.queue_family_index()),
    )
    .unwrap()
}

fn create_render_image_view(render_image: &Arc<StorageImage>) -> Arc<ImageView<StorageImage>> {
    ImageView::new(
        render_image.clone(),
        ImageViewCreateInfo {
            format: Some(render_image.format()),
            usage: ImageUsage {
                transfer_src: true,
                storage: true,
                ..Default::default()
            },
            subresource_range: ImageSubresourceRange {
                aspects: ImageAspects {
                    color: true,
                    ..Default::default()
                },
                mip_levels: 0..1,
                array_layers: 0..1,
            },
            ..Default::default()
        },
    )
    .unwrap()
}

fn create_render_image_set(
    pipeline: &Arc<ComputePipeline>,
    render_image_view: &Arc<ImageView<StorageImage>>,
    camera_buffer: &Arc<CameraBuffer>,
    octree_buffer: &Arc<OctreeBuffer>,
    allocators: &Allocators,
) -> Arc<PersistentDescriptorSet<StandardDescriptorSetAlloc>> {
    let pipeline_layout = pipeline.layout().set_layouts().get(0).unwrap();

    let descriptor_writes = [
        WriteDescriptorSet::image_view(0, render_image_view.clone()),
        WriteDescriptorSet::buffer(1, camera_buffer.clone()),
        WriteDescriptorSet::buffer(2, octree_buffer.clone()),
    ];

    let available_bindings = pipeline_layout
        .bindings()
        .iter()
        .map(|(b, _)| *b)
        .collect::<Vec<_>>();

    PersistentDescriptorSet::new(
        &allocators.descriptor_set,
        pipeline_layout.clone(),
        descriptor_writes
            .into_iter()
            .filter(|w| available_bindings.contains(&w.binding())),
    )
    .unwrap()
}
