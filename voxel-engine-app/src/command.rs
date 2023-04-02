use std::sync::Arc;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, BlitImageInfo, CommandBufferUsage, PrimaryAutoCommandBuffer,
};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAlloc;
use vulkano::descriptor_set::PersistentDescriptorSet;
use vulkano::device::{Device, Queue};
use vulkano::image::{ImageAccess, StorageImage, SwapchainImage};
use vulkano::pipeline::{ComputePipeline, Pipeline, PipelineBindPoint};

pub fn record_command_buffers(
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
                    swapchain_image.dimensions().width().div_ceil(16),
                    swapchain_image.dimensions().height().div_ceil(16),
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
