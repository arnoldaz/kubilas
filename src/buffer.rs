use vulkanalia::{vk::{self, HasBuilder}};
use vulkanalia_vma::{self as vma, Alloc};
use crate::{command::CommandData, vulkan::copy_buffer, vulkan_context::{VulkanContext}};
use anyhow::Result;

pub struct BufferAllocation {
    pub buffer: vk::Buffer,
    pub allocation: vma::Allocation,
}

impl BufferAllocation {
    pub fn _new(buffer: vk::Buffer, allocation: vma::Allocation) -> Self {
        Self { buffer, allocation }
    }

    pub unsafe fn destroy(self, vulkan_context: &VulkanContext) {
        vulkan_context.allocator.destroy_buffer(self.buffer, self.allocation);
    }

    pub unsafe fn allocate_buffer<T: Copy, S: AsRef<[T]>>(data: S, usage_flag: vk::BufferUsageFlags, vulkan_context: &VulkanContext, command_data: &CommandData) -> Result<Self> {
        let allocator = &vulkan_context.allocator;
        let data_slice = data.as_ref();
        let size = (size_of::<T>() * data_slice.len()) as u64;

        let buffer_create_info = vk::BufferCreateInfo::builder()
            .size(size)
            .usage(vk::BufferUsageFlags::TRANSFER_DST | usage_flag)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let allocation_options = vma::AllocationOptions {
            flags: vma::AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE | vma::AllocationCreateFlags::HOST_ACCESS_ALLOW_TRANSFER_INSTEAD,
            usage: vma::MemoryUsage::Auto,
            ..Default::default()
        };

        let (buffer, allocation) = allocator.create_buffer(buffer_create_info, &allocation_options)?;

        // Check if BAR memory is full or not available
        let allocation_info = allocator.get_allocation_info(allocation);
        let memory_properties = allocator.get_memory_properties();

        let memory_type_index = allocation_info.memoryType as usize;
        let memory_type_flags = memory_properties.memory_types[memory_type_index].property_flags;

        let is_bar_memory = memory_type_flags.contains(vk::MemoryPropertyFlags::DEVICE_LOCAL) && memory_type_flags.contains(vk::MemoryPropertyFlags::HOST_VISIBLE);

        // If buffer is not in BAR memory, it means it was placed directly into GPU read only buffer (DEVICE_LOCAL)
        if !is_bar_memory {
            println!("Buffer not in BAR, need staging buffer");

            let buffer_create_info = vk::BufferCreateInfo::builder()
                .size(size)
                .usage(vk::BufferUsageFlags::TRANSFER_SRC)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);

            let allocation_options = vma::AllocationOptions {
                flags: vma::AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE,
                usage: vma::MemoryUsage::Auto,
                ..Default::default()
            };

            let (staging_buffer, staging_allocation) = allocator.create_buffer(buffer_create_info, &allocation_options)?;

            let buffer_data = allocator.map_memory(staging_allocation)? as *mut T;
            buffer_data.copy_from_nonoverlapping(data_slice.as_ptr(), data_slice.len());
            allocator.unmap_memory(staging_allocation);

            // Copy staging buffer to already created DEVICE_LOCAL GPU buffer
            copy_buffer(vulkan_context, command_data, staging_buffer, buffer, size)?;

            allocator.destroy_buffer(staging_buffer, staging_allocation);

            return Ok( Self { buffer, allocation });
        }

        // Copy data to the buffer
        let buffer_data = allocator.map_memory(allocation)? as *mut T;
        buffer_data.copy_from_nonoverlapping(data_slice.as_ptr(), data_slice.len());
        allocator.unmap_memory(allocation);

        Ok( Self { buffer, allocation })
    }
}