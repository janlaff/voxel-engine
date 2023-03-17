use crate::device_selection::find_device;
use std::sync::Arc;
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType};
use vulkano::device::{
    Device, DeviceCreateInfo, DeviceExtensions, Features, Queue, QueueCreateInfo,
};
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::swapchain::Surface;
use vulkano::VulkanLibrary;
use vulkano_win::VkSurfaceBuild;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

pub struct Context {
    pub instance: Arc<Instance>,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub surface: Arc<Surface>,
}

impl Context {
    pub fn new(event_loop: &EventLoop<()>, window_builder: WindowBuilder) -> Self {
        let library = VulkanLibrary::new().expect("Failed to load vulkan library");

        let enabled_extensions = vulkano_win::required_extensions(&library);
        let enabled_layers = ["VK_LAYER_LUNARG_monitor"];

        let instance = Instance::new(
            library,
            InstanceCreateInfo {
                enabled_extensions,
                enabled_layers: enabled_layers
                    .iter()
                    .map(|layer| String::from(*layer))
                    .collect(),
                ..Default::default()
            },
        )
        .expect("Failed to create instance");

        let surface = window_builder
            .build_vk_surface(&event_loop, instance.clone())
            .unwrap();

        let (physical, device, queue) = find_device(&instance, &surface);

        Self {
            instance,
            device,
            queue,
            surface,
        }
    }

    pub fn window(&'_ self) -> &'_ Window {
        self.surface
            .object()
            .unwrap()
            .downcast_ref::<Window>()
            .unwrap()
    }
}
