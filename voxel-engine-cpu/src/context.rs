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

        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };

        let device_features = Features {
            vulkan_memory_model: true,
            shader_int8: true,
            shader_int16: true,
            ..Default::default()
        };

        let (physical, queue_family_index) =
            select_physical_device(&instance, &surface, &device_extensions, &device_features);
        let (device, mut queues) = Device::new(
            physical,
            DeviceCreateInfo {
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                enabled_extensions: device_extensions,
                enabled_features: device_features,
                ..Default::default()
            },
        )
        .expect("Failed to create logical device");
        let queue = queues.next().unwrap();

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

fn select_physical_device(
    instance: &Arc<Instance>,
    surface: &Arc<Surface>,
    device_extensions: &DeviceExtensions,
    device_features: &Features,
) -> (Arc<PhysicalDevice>, u32) {
    instance
        .enumerate_physical_devices()
        .expect("Failed to enumerate physical devices")
        .filter(|p| {
            p.supported_extensions().contains(device_extensions)
                && p.supported_features().contains(device_features)
        })
        .filter_map(|p| {
            p.queue_family_properties()
                .iter()
                .enumerate()
                .position(|(i, q)| {
                    q.queue_flags.graphics
                        && q.queue_flags.compute
                        && p.surface_support(i as u32, &surface).unwrap_or(false)
                })
                .map(|q| (p, q as u32))
        })
        .min_by_key(|(p, _)| match p.properties().device_type {
            PhysicalDeviceType::DiscreteGpu => 0,
            PhysicalDeviceType::IntegratedGpu => 1,
            PhysicalDeviceType::VirtualGpu => 2,
            PhysicalDeviceType::Cpu => 3,
            _ => 4,
        })
        .expect("No suitable physical devices available")
}
