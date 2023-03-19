use std::sync::Arc;
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType};
use vulkano::device::{
    Device, DeviceCreateInfo, DeviceExtensions, Features, Queue, QueueCreateInfo,
};
use vulkano::instance::Instance;
use vulkano::swapchain::Surface;

const DEVICE_EXTENSIONS: DeviceExtensions = DeviceExtensions {
    khr_swapchain: true,
    ..DeviceExtensions::empty()
};

const DEVICE_FEATURES: Features = Features {
    vulkan_memory_model: true,
    shader_int8: true,
    shader_int16: true,
    ..Features::empty()
};

pub struct GpuModel {
    pub physical: Arc<PhysicalDevice>,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
}

pub fn find_gpu_model(instance: &Arc<Instance>, surface: &Arc<Surface>) -> GpuModel {
    let (physical, queue_family_index) = instance
        .enumerate_physical_devices()
        .expect("Failed to enumerate physical devices")
        .filter(|p| physical_device_supported(p))
        .filter_map(|p| find_queue_family(p, surface))
        .min_by_key(|(p, _)| rate_physical_device(p))
        .expect("No suitable physical devices available");

    let (device, queue) = Device::new(
        physical.clone(),
        DeviceCreateInfo {
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            enabled_extensions: DEVICE_EXTENSIONS,
            enabled_features: DEVICE_FEATURES,
            ..Default::default()
        },
    )
    .map(|(device, mut queues)| (device, queues.next().unwrap()))
    .expect("Failed to create logical device");

    GpuModel {
        physical,
        device,
        queue,
    }
}

fn physical_device_supported(physical: &Arc<PhysicalDevice>) -> bool {
    let extensions_supported = physical.supported_extensions().contains(&DEVICE_EXTENSIONS);
    let features_supported = physical.supported_features().contains(&DEVICE_FEATURES);

    extensions_supported && features_supported
}

fn find_queue_family(
    physical: Arc<PhysicalDevice>,
    surface: &Arc<Surface>,
) -> Option<(Arc<PhysicalDevice>, u32)> {
    physical
        .queue_family_properties()
        .iter()
        .enumerate()
        .position(|(queue_family_index, queue_family)| {
            queue_family.queue_flags.graphics
                && queue_family.queue_flags.compute
                && physical
                    .surface_support(queue_family_index as u32, &surface)
                    .unwrap_or(false)
        })
        .map(|queue_family_index| (physical, queue_family_index as u32))
}

fn rate_physical_device(physical: &Arc<PhysicalDevice>) -> u32 {
    match physical.properties().device_type {
        PhysicalDeviceType::DiscreteGpu => 0,
        PhysicalDeviceType::IntegratedGpu => 1,
        PhysicalDeviceType::VirtualGpu => 2,
        PhysicalDeviceType::Cpu => 3,
        _ => 4,
    }
}
