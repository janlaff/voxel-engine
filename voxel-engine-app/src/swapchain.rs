use std::sync::Arc;
use vulkano::device::Device;
use vulkano::format::Format;
use vulkano::image::{ImageUsage, SwapchainImage};
use vulkano::swapchain::{PresentMode, Surface, Swapchain, SwapchainCreateInfo};
use winit::dpi::PhysicalSize;

pub fn create_swapchain(
    device: &Arc<Device>,
    surface: &Arc<Surface>,
    screen_size: PhysicalSize<u32>,
) -> (Arc<Swapchain>, Vec<Arc<SwapchainImage>>) {
    let caps = device
        .physical_device()
        .surface_capabilities(surface, Default::default())
        .expect("Failed to get surface capabilities");

    let composite_alpha = caps.supported_composite_alpha.into_iter().next().unwrap();

    let format = *device
        .physical_device()
        .surface_formats(surface, Default::default())
        .unwrap()
        .iter()
        .find(|(f, c)| *f == Format::B8G8R8A8_SRGB)
        .unwrap();

    Swapchain::new(
        device.clone(),
        surface.clone(),
        SwapchainCreateInfo {
            min_image_count: caps.min_image_count + 1,
            image_format: Some(format.0),
            image_color_space: format.1,
            image_extent: screen_size.into(),
            image_usage: ImageUsage::TRANSFER_DST,
            present_mode: PresentMode::Mailbox,
            clipped: false,
            composite_alpha,
            ..Default::default()
        },
    )
    .unwrap()
}
