use std::collections::HashSet;
use std::ffi::CStr;
use std::os::raw::c_void;
use anyhow::{anyhow, Result};
use log::{debug, error, info, trace, warn};
use winit::window::Window;
use vulkanalia::loader::{LIBRARY, LibloadingLoader};
use vulkanalia::{Device, Entry, Instance, vk, window as vk_window};
use vulkanalia::vk::{DeviceV1_0, EntryV1_0, ExtDebugUtilsExtensionInstanceCommands, Handle, HasBuilder, InstanceV1_0, KhrSurfaceExtensionInstanceCommands};
use vulkanalia_vma::{self as vma};

pub struct VulkanContext {
    pub _entry: Entry,
    pub instance: Instance,
    pub device: Device,
    
    pub surface: vk::SurfaceKHR,
    pub messenger: vk::DebugUtilsMessengerEXT,
    pub physical_device: vk::PhysicalDevice,
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,

    pub allocator: vma::Allocator,
}

impl VulkanContext {
    const VALIDATION_ENABLED: bool = cfg!(debug_assertions);
    const VALIDATION_LAYER: vk::ExtensionName = vk::ExtensionName::from_bytes(b"VK_LAYER_KHRONOS_validation");
    const DEVICE_EXTENSIONS: &[vk::ExtensionName] = &[vk::KHR_SWAPCHAIN_EXTENSION.name];

    pub unsafe fn new(window: &Window) -> Result<Self> {
        let loader = LibloadingLoader::new(LIBRARY)?;
        let entry = Entry::new(loader).map_err(|b| anyhow!("{}", b))?;

        let (instance, messenger) = Self::create_instance(window, &entry)?;
        let surface = vk_window::create_surface(&instance, &window, &window)?;
        let physical_device = Self::pick_physical_device(&instance, surface)?;
        let (device, graphics_queue, present_queue) = Self::create_logical_device(&instance, physical_device, surface)?;

        let allocator_options = vma::AllocatorOptions::new(&instance, &device, physical_device);
        let allocator = vma::Allocator::new(&allocator_options)?;

        Ok(Self { _entry: entry, instance, device, surface, messenger, physical_device, graphics_queue, present_queue, allocator })
    }

    pub unsafe fn destroy(self) {
        drop(self.allocator);

        self.device.destroy_device(None);
        self.instance.destroy_surface_khr(self.surface, None);
    
        if Self::VALIDATION_ENABLED {
            self.instance.destroy_debug_utils_messenger_ext(self.messenger, None);
        }
    
        self.instance.destroy_instance(None);
    }

    unsafe fn create_instance(window: &Window, entry: &Entry) -> Result<(Instance, vk::DebugUtilsMessengerEXT)> {
        let application_info = vk::ApplicationInfo::builder()
            .application_name(b"Kubilas\0")
            .application_version(vk::make_version(1, 0, 0))
            .engine_name(b"Kubilas\0")
            .engine_version(vk::make_version(1, 0, 0))
            .api_version(vk::make_version(1, 4, 0));

        let mut extensions = vk_window::get_required_instance_extensions(window)
            .iter()
            .map(|e| e.as_ptr())
            .collect::<Vec<_>>();

        if Self::VALIDATION_ENABLED {
            extensions.push(vk::EXT_DEBUG_UTILS_EXTENSION.name.as_ptr());
        }

        let available_layers = entry
            .enumerate_instance_layer_properties()?
            .iter()
            .map(|l| l.layer_name)
            .collect::<HashSet<_>>();

        if Self::VALIDATION_ENABLED && !available_layers.contains(&Self::VALIDATION_LAYER) {
            return Err(anyhow!("Validation layer requested but not supported"));
        }

        let layers = if Self::VALIDATION_ENABLED {
            vec![Self::VALIDATION_LAYER.as_ptr()]
        } else {
            Vec::new()
        };

        let mut info = vk::InstanceCreateInfo::builder()
            .application_info(&application_info)
            .enabled_layer_names(&layers)
            .enabled_extension_names(&extensions);

        let mut debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::all())
            .message_type(vk::DebugUtilsMessageTypeFlagsEXT::GENERAL | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE)
            .user_callback(Some(Self::debug_callback));

        let verbose_value: [u32; 1] = [1];
        let layer_settings = vk::LayerSettingEXT::builder()
            .layer_name(b"VK_LAYER_KHRONOS_validation\0")
            .setting_name(b"validate_sync\0")
            .values_bool32(&verbose_value);

        let layer_settings_vec = [layer_settings];
        let mut layer_settings_create_info = vk::LayerSettingsCreateInfoEXT::builder()
            .settings(&layer_settings_vec);

        let gpuav_enable: [u32; 1] = [1];
        let gpuav_settings = vk::LayerSettingEXT::builder()
            .layer_name(b"VK_LAYER_KHRONOS_validation\0")
            .setting_name(b"gpuav_enable\0")
            .values_bool32(&gpuav_enable);

        let gpuav_settings_vec = [gpuav_settings];
        let mut gpuav_settings_create_info = vk::LayerSettingsCreateInfoEXT::builder()
            .settings(&gpuav_settings_vec);

        if Self::VALIDATION_ENABLED {
            info = info.push_next(&mut layer_settings_create_info);
            info = info.push_next(&mut gpuav_settings_create_info);
            info = info.push_next(&mut debug_info);
        }

        let instance = entry.create_instance(&info, None)?;

        let mut messenger = vk::DebugUtilsMessengerEXT::null();
        if Self::VALIDATION_ENABLED {
            let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
                .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::all())
                .message_type(vk::DebugUtilsMessageTypeFlagsEXT::GENERAL | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE)
                .user_callback(Some(Self::debug_callback));

            messenger = instance.create_debug_utils_messenger_ext(&debug_info, None)?;
        }

        Ok((instance, messenger))
    }

    unsafe fn pick_physical_device(instance: &Instance, surface: vk::SurfaceKHR) -> Result<vk::PhysicalDevice> {
        for physical_device in instance.enumerate_physical_devices()? {
            let properties = instance.get_physical_device_properties(physical_device);

            if let Err(error) = Self::check_physical_device(instance, physical_device, surface) {
                warn!("Skipping physical device (`{}`): {}", properties.device_name, error);
            } else {
                info!("Selected physical device (`{}`)", properties.device_name);
                return Ok(physical_device);
            }
        }

        Err(anyhow!("Failed to find suitable physical device."))
    }

    unsafe fn check_physical_device(instance: &Instance, physical_device: vk::PhysicalDevice, surface: vk::SurfaceKHR) -> Result<()> {
        _ = Self::get_queue_family_indices(instance, physical_device, surface)?;
        Self::check_physical_device_extensions(instance, physical_device)?;

        let formats = instance.get_physical_device_surface_formats_khr(physical_device, surface)?;
        let present_modes = instance.get_physical_device_surface_present_modes_khr(physical_device, surface)?;
        if formats.is_empty() || present_modes.is_empty() {
            return Err(anyhow!("Insufficient swapchain support"));
        }

        let features = instance.get_physical_device_features(physical_device);
        if features.sampler_anisotropy != vk::TRUE {
            return Err(anyhow!("No sampler anisotropy"));
        }

        Ok(())
    }

    unsafe fn check_physical_device_extensions(instance: &Instance, physical_device: vk::PhysicalDevice) -> Result<()> {
        let extensions = instance
            .enumerate_device_extension_properties(physical_device, None)?
            .iter()
            .map(|e| e.extension_name)
            .collect::<HashSet<_>>();

        if Self::DEVICE_EXTENSIONS.iter().all(|e| extensions.contains(e)) {
            Ok(())
        } else {
            Err(anyhow!("Missing required device extensions"))
        }
    }

    unsafe fn create_logical_device(instance: &Instance, physical_device: vk::PhysicalDevice, surface: vk::SurfaceKHR) -> Result<(Device, vk::Queue, vk::Queue)> {
        let (graphics, present) = Self::get_queue_family_indices(instance, physical_device, surface)?;

        let mut unique_indices = HashSet::new();
        unique_indices.insert(graphics);
        unique_indices.insert(present);

        let queue_infos = unique_indices
            .iter()
            .map(|i| {
                vk::DeviceQueueCreateInfo::builder()
                    .queue_family_index(*i)
                    .queue_priorities(&[1.0])
            })
            .collect::<Vec<_>>();

        let layers = if Self::VALIDATION_ENABLED {
            vec![Self::VALIDATION_LAYER.as_ptr()]
        } else {
            vec![]
        };

        let extensions = Self::DEVICE_EXTENSIONS
            .iter()
            .map(|n| n.as_ptr())
            .collect::<Vec<_>>();

        let mut features12 = vk::PhysicalDeviceVulkan12Features::builder()
            .runtime_descriptor_array(true)
            .descriptor_binding_partially_bound(true)
            .descriptor_binding_sampled_image_update_after_bind(true)
            .descriptor_binding_storage_image_update_after_bind(true)
            .descriptor_binding_uniform_buffer_update_after_bind(true)
            .descriptor_binding_variable_descriptor_count(true)
            .shader_sampled_image_array_non_uniform_indexing(true)
            .shader_storage_buffer_array_non_uniform_indexing(true)
            .shader_uniform_buffer_array_non_uniform_indexing(true)
            .descriptor_indexing(true);

        let mut features13 = vk::PhysicalDeviceVulkan13Features::builder()
            .dynamic_rendering(true)
            .synchronization2(true);

        let mut features2 = vk::PhysicalDeviceFeatures2::builder()
            .features(
                vk::PhysicalDeviceFeatures::builder()
                    .sampler_anisotropy(true)
            )
            .push_next(&mut features12)
            .push_next(&mut features13);

        let info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_infos)
            .enabled_layer_names(&layers)
            .enabled_extension_names(&extensions)
            .push_next(&mut features2);

        let device = instance.create_device(physical_device, &info, None)?;
        
        let graphics_queue = device.get_device_queue(graphics, 0);
        let present_queue = device.get_device_queue(present, 0);

        Ok((device, graphics_queue, present_queue))
    }

    unsafe fn get_queue_family_indices(instance: &Instance, physical_device: vk::PhysicalDevice, surface: vk::SurfaceKHR) -> Result<(u32, u32)> {
        let properties = instance.get_physical_device_queue_family_properties(physical_device);
        let graphics = properties
            .iter()
            .position(|p| p.queue_flags.contains(vk::QueueFlags::GRAPHICS))
            .map(|i| i as u32);

        let mut present = None;
        for index in 0..properties.len() {
            if instance.get_physical_device_surface_support_khr(physical_device, index as u32, surface)? {
                present = Some(index as u32);
                break;
            }
        }

        if let (Some(graphics), Some(present)) = (graphics, present) {
            Ok((graphics, present))
        } else {
            Err(anyhow!("Missing required queue families"))
        }
    }

    pub unsafe fn queue_family_indices(&self) -> Result<(u32, u32)> {
        Self::get_queue_family_indices(&self.instance, self.physical_device, self.surface)
    }

    extern "system" fn debug_callback(
        severity: vk::DebugUtilsMessageSeverityFlagsEXT,
        type_: vk::DebugUtilsMessageTypeFlagsEXT,
        data: *const vk::DebugUtilsMessengerCallbackDataEXT,
        _: *mut c_void,
    ) -> vk::Bool32 {
        let data = unsafe { *data };
        let message = unsafe { CStr::from_ptr(data.message) }.to_string_lossy();

        if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::ERROR {
            error!("({:?}) {}", type_, message);
        } else if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::WARNING {
            warn!("({:?}) {}", type_, message);
        } else if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::INFO {
            debug!("({:?}) {}", type_, message);
        } else {
            trace!("({:?}) {}", type_, message);
        }

        vk::FALSE
    }
}
