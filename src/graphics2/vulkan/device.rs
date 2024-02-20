use std::ops::Deref;
use std::sync::Arc;

use ash::extensions::khr;
use ash::vk;

use super::instance::Instance;
use super::Error;

pub struct Device {
    handle: ash::Device,

    physical_device: Arc<PhysicalDevice>,
}

impl Device {
    pub fn new(
        instance: &Instance,
        physical_device: Arc<PhysicalDevice>,
        create_info: &vk::DeviceCreateInfo,
    ) -> Result<Self, Error> {
        let device = unsafe { instance.create_device(physical_device.handle, create_info, None)? };

        Ok(Self {
            handle: device,
            physical_device,
        })
    }

    /// Returns a reference to this device [`PhysicalDevice`].
    pub fn physical_device(&self) -> &PhysicalDevice {
        &self.physical_device
    }
}

impl Deref for Device {
    type Target = ash::Device;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

#[derive(Debug, Clone)]
pub struct PhysicalDevice {
    pub handle: vk::PhysicalDevice,

    extension_properties: Vec<ExtensionProperties>,
    properties: PhysicalDeviceProperties,
    features: PhysicalDeviceFeatures,
    queue_family_properties: Vec<vk::QueueFamilyProperties>,
}

impl PhysicalDevice {
    pub fn from_handle(
        instance: &ash::Instance,
        handle: vk::PhysicalDevice,
    ) -> Result<Self, Error> {
        let extensions = unsafe { instance.enumerate_device_extension_properties(handle)? }
            .into_iter()
            .map(ExtensionProperties::from)
            .collect();
        let properties = unsafe { instance.get_physical_device_properties(handle) }.into();
        let features = unsafe { instance.get_physical_device_features(handle) }.into();
        let queue_family_properties =
            unsafe { instance.get_physical_device_queue_family_properties(handle) };

        Ok(Self {
            handle,
            extension_properties: extensions,
            properties,
            features,
            queue_family_properties,
        })
    }

    /// Returns an iterator over the [`PhysicalDevice`] [extension properties](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkExtensionProperties.html).
    pub fn extension_properties(&self) -> impl Iterator<Item = &ExtensionProperties> {
        self.extension_properties.iter()
    }

    /// Returns the supported features by this device.
    pub fn features(&self) -> &PhysicalDeviceFeatures {
        &self.features
    }

    /// Returns this device properties.
    pub fn properties(&self) -> &PhysicalDeviceProperties {
        &self.properties
    }

    /// Returns an iterator over the [`PhysicalDevice`] [extension properties](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkQueueFamilyProperties.html).
    pub fn queue_family_properties(&self) -> impl Iterator<Item = vk::QueueFamilyProperties> + '_ {
        self.queue_family_properties.iter().copied()
    }

    /// Checks whether this physical device supports the specified extensions.
    pub fn supports_extensions(&self, extensions: Vec<String>) -> bool {
        extensions.iter().all(|extension| {
            self.extension_properties()
                .any(|ext| ext.extension_name == *extension)
        })
    }

    /// Checks if this PhysicalDevice supports the surface
    pub fn supports_surface(
        &self,
        queue_family_index: u32,
        surface_loader: &khr::Surface,
        surface: vk::SurfaceKHR,
    ) -> Result<bool, Error> {
        Ok(unsafe {
            surface_loader.get_physical_device_surface_support(
                self.handle,
                queue_family_index,
                surface,
            )?
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct PhysicalDeviceFeatures {
    pub robust_buffer_access: bool,
    pub full_draw_index_uint32: bool,
    pub image_cube_array: bool,
    pub independent_blend: bool,
    pub geometry_shader: bool,
    pub tessellation_shader: bool,
    pub sample_rate_shading: bool,
    pub dual_src_blend: bool,
    pub logic_op: bool,
    pub multi_draw_indirect: bool,
    pub draw_indirect_first_instance: bool,
    pub depth_clamp: bool,
    pub depth_bias_clamp: bool,
    pub fill_mode_non_solid: bool,
    pub depth_bounds: bool,
    pub wide_lines: bool,
    pub large_points: bool,
    pub alpha_to_one: bool,
    pub multi_viewport: bool,
    pub sampler_anisotropy: bool,
    pub texture_compression_etc2: bool,
    pub texture_compression_astc_ldr: bool,
    pub texture_compression_bc: bool,
    pub occlusion_query_precise: bool,
    pub pipeline_statistics_query: bool,
    pub vertex_pipeline_stores_and_atomics: bool,
    pub fragment_stores_and_atomics: bool,
    pub shader_tessellation_and_geometry_point_size: bool,
    pub shader_image_gather_extended: bool,
    pub shader_storage_image_extended_formats: bool,
    pub shader_storage_image_multisample: bool,
    pub shader_storage_image_read_without_format: bool,
    pub shader_storage_image_write_without_format: bool,
    pub shader_uniform_buffer_array_dynamic_indexing: bool,
    pub shader_sampled_image_array_dynamic_indexing: bool,
    pub shader_storage_buffer_array_dynamic_indexing: bool,
    pub shader_storage_image_array_dynamic_indexing: bool,
    pub shader_clip_distance: bool,
    pub shader_cull_distance: bool,
    pub shader_float64: bool,
    pub shader_int64: bool,
    pub shader_int16: bool,
    pub shader_resource_residency: bool,
    pub shader_resource_min_lod: bool,
    pub sparse_binding: bool,
    pub sparse_residency_buffer: bool,
    pub sparse_residency_image2_d: bool,
    pub sparse_residency_image3_d: bool,
    pub sparse_residency2_samples: bool,
    pub sparse_residency4_samples: bool,
    pub sparse_residency8_samples: bool,
    pub sparse_residency16_samples: bool,
    pub sparse_residency_aliased: bool,
    pub variable_multisample_rate: bool,
    pub inherited_queries: bool,
}

impl PhysicalDeviceFeatures {
    pub fn contains(&self, other: &Self) -> bool {
        (self.robust_buffer_access || !other.robust_buffer_access)
            && (self.full_draw_index_uint32 || !other.full_draw_index_uint32)
            && (self.image_cube_array || !other.image_cube_array)
            && (self.independent_blend || !other.independent_blend)
            && (self.geometry_shader || !other.geometry_shader)
            && (self.tessellation_shader || !other.tessellation_shader)
            && (self.sample_rate_shading || !other.sample_rate_shading)
            && (self.dual_src_blend || !other.dual_src_blend)
            && (self.logic_op || !other.logic_op)
            && (self.multi_draw_indirect || !other.multi_draw_indirect)
            && (self.draw_indirect_first_instance || !other.draw_indirect_first_instance)
            && (self.depth_clamp || !other.depth_clamp)
            && (self.depth_bias_clamp || !other.depth_bias_clamp)
            && (self.fill_mode_non_solid || !other.fill_mode_non_solid)
            && (self.depth_bounds || !other.depth_bounds)
            && (self.wide_lines || !other.wide_lines)
            && (self.large_points || !other.large_points)
            && (self.alpha_to_one || !other.alpha_to_one)
            && (self.multi_viewport || !other.multi_viewport)
            && (self.sampler_anisotropy || !other.sampler_anisotropy)
            && (self.texture_compression_etc2 || !other.texture_compression_etc2)
            && (self.texture_compression_astc_ldr || !other.texture_compression_astc_ldr)
            && (self.texture_compression_bc || !other.texture_compression_bc)
            && (self.occlusion_query_precise || !other.occlusion_query_precise)
            && (self.pipeline_statistics_query || !other.pipeline_statistics_query)
            && (self.vertex_pipeline_stores_and_atomics
                || !other.vertex_pipeline_stores_and_atomics)
            && (self.fragment_stores_and_atomics || !other.fragment_stores_and_atomics)
            && (self.shader_tessellation_and_geometry_point_size
                || !other.shader_tessellation_and_geometry_point_size)
            && (self.shader_image_gather_extended || !other.shader_image_gather_extended)
            && (self.shader_storage_image_extended_formats
                || !other.shader_storage_image_extended_formats)
            && (self.shader_storage_image_multisample || !other.shader_storage_image_multisample)
            && (self.shader_storage_image_read_without_format
                || !other.shader_storage_image_read_without_format)
            && (self.shader_storage_image_write_without_format
                || !other.shader_storage_image_write_without_format)
            && (self.shader_uniform_buffer_array_dynamic_indexing
                || !other.shader_uniform_buffer_array_dynamic_indexing)
            && (self.shader_sampled_image_array_dynamic_indexing
                || !other.shader_sampled_image_array_dynamic_indexing)
            && (self.shader_storage_buffer_array_dynamic_indexing
                || !other.shader_storage_buffer_array_dynamic_indexing)
            && (self.shader_storage_image_array_dynamic_indexing
                || !other.shader_storage_image_array_dynamic_indexing)
            && (self.shader_clip_distance || !other.shader_clip_distance)
            && (self.shader_cull_distance || !other.shader_cull_distance)
            && (self.shader_float64 || !other.shader_float64)
            && (self.shader_int64 || !other.shader_int64)
            && (self.shader_int16 || !other.shader_int16)
            && (self.shader_resource_residency || !other.shader_resource_residency)
            && (self.shader_resource_min_lod || !other.shader_resource_min_lod)
            && (self.sparse_binding || !other.sparse_binding)
            && (self.sparse_residency_buffer || !other.sparse_residency_buffer)
            && (self.sparse_residency_image2_d || !other.sparse_residency_image2_d)
            && (self.sparse_residency_image3_d || !other.sparse_residency_image3_d)
            && (self.sparse_residency2_samples || !other.sparse_residency2_samples)
            && (self.sparse_residency4_samples || !other.sparse_residency4_samples)
            && (self.sparse_residency8_samples || !other.sparse_residency8_samples)
            && (self.sparse_residency16_samples || !other.sparse_residency16_samples)
            && (self.sparse_residency_aliased || !other.sparse_residency_aliased)
            && (self.variable_multisample_rate || !other.variable_multisample_rate)
            && (self.inherited_queries || !other.inherited_queries)
    }
}

impl From<vk::PhysicalDeviceFeatures> for PhysicalDeviceFeatures {
    fn from(value: vk::PhysicalDeviceFeatures) -> Self {
        Self {
            robust_buffer_access: from_bool32(value.robust_buffer_access),
            full_draw_index_uint32: from_bool32(value.full_draw_index_uint32),
            image_cube_array: from_bool32(value.image_cube_array),
            independent_blend: from_bool32(value.independent_blend),
            geometry_shader: from_bool32(value.geometry_shader),
            tessellation_shader: from_bool32(value.tessellation_shader),
            sample_rate_shading: from_bool32(value.sample_rate_shading),
            dual_src_blend: from_bool32(value.dual_src_blend),
            logic_op: from_bool32(value.logic_op),
            multi_draw_indirect: from_bool32(value.multi_draw_indirect),
            draw_indirect_first_instance: from_bool32(value.draw_indirect_first_instance),
            depth_clamp: from_bool32(value.depth_clamp),
            depth_bias_clamp: from_bool32(value.depth_bias_clamp),
            fill_mode_non_solid: from_bool32(value.fill_mode_non_solid),
            depth_bounds: from_bool32(value.depth_bounds),
            wide_lines: from_bool32(value.wide_lines),
            large_points: from_bool32(value.large_points),
            alpha_to_one: from_bool32(value.alpha_to_one),
            multi_viewport: from_bool32(value.multi_viewport),
            sampler_anisotropy: from_bool32(value.sampler_anisotropy),
            texture_compression_etc2: from_bool32(value.texture_compression_etc2),
            texture_compression_astc_ldr: from_bool32(value.texture_compression_astc_ldr),
            texture_compression_bc: from_bool32(value.texture_compression_bc),
            occlusion_query_precise: from_bool32(value.occlusion_query_precise),
            pipeline_statistics_query: from_bool32(value.pipeline_statistics_query),
            vertex_pipeline_stores_and_atomics: from_bool32(
                value.vertex_pipeline_stores_and_atomics,
            ),
            fragment_stores_and_atomics: from_bool32(value.fragment_stores_and_atomics),
            shader_tessellation_and_geometry_point_size: from_bool32(
                value.shader_tessellation_and_geometry_point_size,
            ),
            shader_image_gather_extended: from_bool32(value.shader_image_gather_extended),
            shader_storage_image_extended_formats: from_bool32(
                value.shader_storage_image_extended_formats,
            ),
            shader_storage_image_multisample: from_bool32(value.shader_storage_image_multisample),
            shader_storage_image_read_without_format: from_bool32(
                value.shader_storage_image_read_without_format,
            ),
            shader_storage_image_write_without_format: from_bool32(
                value.shader_storage_image_write_without_format,
            ),
            shader_uniform_buffer_array_dynamic_indexing: from_bool32(
                value.shader_uniform_buffer_array_dynamic_indexing,
            ),
            shader_sampled_image_array_dynamic_indexing: from_bool32(
                value.shader_sampled_image_array_dynamic_indexing,
            ),
            shader_storage_buffer_array_dynamic_indexing: from_bool32(
                value.shader_storage_buffer_array_dynamic_indexing,
            ),
            shader_storage_image_array_dynamic_indexing: from_bool32(
                value.shader_storage_image_array_dynamic_indexing,
            ),
            shader_clip_distance: from_bool32(value.shader_clip_distance),
            shader_cull_distance: from_bool32(value.shader_cull_distance),
            shader_float64: from_bool32(value.shader_float64),
            shader_int64: from_bool32(value.shader_int64),
            shader_int16: from_bool32(value.shader_int16),
            shader_resource_residency: from_bool32(value.shader_resource_residency),
            shader_resource_min_lod: from_bool32(value.shader_resource_min_lod),
            sparse_binding: from_bool32(value.sparse_binding),
            sparse_residency_buffer: from_bool32(value.sparse_residency_buffer),
            sparse_residency_image2_d: from_bool32(value.sparse_residency_image2_d),
            sparse_residency_image3_d: from_bool32(value.sparse_residency_image3_d),
            sparse_residency2_samples: from_bool32(value.sparse_residency2_samples),
            sparse_residency4_samples: from_bool32(value.sparse_residency4_samples),
            sparse_residency8_samples: from_bool32(value.sparse_residency8_samples),
            sparse_residency16_samples: from_bool32(value.sparse_residency16_samples),
            sparse_residency_aliased: from_bool32(value.sparse_residency_aliased),
            variable_multisample_rate: from_bool32(value.variable_multisample_rate),
            inherited_queries: from_bool32(value.inherited_queries),
        }
    }
}

impl From<PhysicalDeviceFeatures> for vk::PhysicalDeviceFeatures {
    fn from(value: PhysicalDeviceFeatures) -> Self {
        Self {
            robust_buffer_access: into_bool32(value.robust_buffer_access),
            full_draw_index_uint32: into_bool32(value.full_draw_index_uint32),
            image_cube_array: into_bool32(value.image_cube_array),
            independent_blend: into_bool32(value.independent_blend),
            geometry_shader: into_bool32(value.geometry_shader),
            tessellation_shader: into_bool32(value.tessellation_shader),
            sample_rate_shading: into_bool32(value.sample_rate_shading),
            dual_src_blend: into_bool32(value.dual_src_blend),
            logic_op: into_bool32(value.logic_op),
            multi_draw_indirect: into_bool32(value.multi_draw_indirect),
            draw_indirect_first_instance: into_bool32(value.draw_indirect_first_instance),
            depth_clamp: into_bool32(value.depth_clamp),
            depth_bias_clamp: into_bool32(value.depth_bias_clamp),
            fill_mode_non_solid: into_bool32(value.fill_mode_non_solid),
            depth_bounds: into_bool32(value.depth_bounds),
            wide_lines: into_bool32(value.wide_lines),
            large_points: into_bool32(value.large_points),
            alpha_to_one: into_bool32(value.alpha_to_one),
            multi_viewport: into_bool32(value.multi_viewport),
            sampler_anisotropy: into_bool32(value.sampler_anisotropy),
            texture_compression_etc2: into_bool32(value.texture_compression_etc2),
            texture_compression_astc_ldr: into_bool32(value.texture_compression_astc_ldr),
            texture_compression_bc: into_bool32(value.texture_compression_bc),
            occlusion_query_precise: into_bool32(value.occlusion_query_precise),
            pipeline_statistics_query: into_bool32(value.pipeline_statistics_query),
            vertex_pipeline_stores_and_atomics: into_bool32(
                value.vertex_pipeline_stores_and_atomics,
            ),
            fragment_stores_and_atomics: into_bool32(value.fragment_stores_and_atomics),
            shader_tessellation_and_geometry_point_size: into_bool32(
                value.shader_tessellation_and_geometry_point_size,
            ),
            shader_image_gather_extended: into_bool32(value.shader_image_gather_extended),
            shader_storage_image_extended_formats: into_bool32(
                value.shader_storage_image_extended_formats,
            ),
            shader_storage_image_multisample: into_bool32(value.shader_storage_image_multisample),
            shader_storage_image_read_without_format: into_bool32(
                value.shader_storage_image_read_without_format,
            ),
            shader_storage_image_write_without_format: into_bool32(
                value.shader_storage_image_write_without_format,
            ),
            shader_uniform_buffer_array_dynamic_indexing: into_bool32(
                value.shader_uniform_buffer_array_dynamic_indexing,
            ),
            shader_sampled_image_array_dynamic_indexing: into_bool32(
                value.shader_sampled_image_array_dynamic_indexing,
            ),
            shader_storage_buffer_array_dynamic_indexing: into_bool32(
                value.shader_storage_buffer_array_dynamic_indexing,
            ),
            shader_storage_image_array_dynamic_indexing: into_bool32(
                value.shader_storage_image_array_dynamic_indexing,
            ),
            shader_clip_distance: into_bool32(value.shader_clip_distance),
            shader_cull_distance: into_bool32(value.shader_cull_distance),
            shader_float64: into_bool32(value.shader_float64),
            shader_int64: into_bool32(value.shader_int64),
            shader_int16: into_bool32(value.shader_int16),
            shader_resource_residency: into_bool32(value.shader_resource_residency),
            shader_resource_min_lod: into_bool32(value.shader_resource_min_lod),
            sparse_binding: into_bool32(value.sparse_binding),
            sparse_residency_buffer: into_bool32(value.sparse_residency_buffer),
            sparse_residency_image2_d: into_bool32(value.sparse_residency_image2_d),
            sparse_residency_image3_d: into_bool32(value.sparse_residency_image3_d),
            sparse_residency2_samples: into_bool32(value.sparse_residency2_samples),
            sparse_residency4_samples: into_bool32(value.sparse_residency4_samples),
            sparse_residency8_samples: into_bool32(value.sparse_residency8_samples),
            sparse_residency16_samples: into_bool32(value.sparse_residency16_samples),
            sparse_residency_aliased: into_bool32(value.sparse_residency_aliased),
            variable_multisample_rate: into_bool32(value.variable_multisample_rate),
            inherited_queries: into_bool32(value.inherited_queries),
        }
    }
}

#[inline]
fn into_bool32(boolean: bool) -> u32 {
    match boolean {
        true => 1,
        false => 0,
    }
}

#[inline]
fn from_bool32(bool32: vk::Bool32) -> bool {
    if bool32 == 1 {
        true
    } else {
        false
    }
}

#[inline]
fn from_i8_array(string: &[i8]) -> String {
    let bytes = bytemuck::cast_slice(string);
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[0..end]).into()
}

#[derive(Debug, Clone)]
pub struct PhysicalDeviceProperties {
    pub api_version: u32,
    pub driver_version: u32,
    pub vendor_id: u32,
    pub device_id: u32,
    pub device_type: vk::PhysicalDeviceType,
    pub device_name: String,
    pub pipeline_cache_uuid: [u8; 16],
    pub limits: vk::PhysicalDeviceLimits,
    pub sparse_properties: vk::PhysicalDeviceSparseProperties,
}

impl From<vk::PhysicalDeviceProperties> for PhysicalDeviceProperties {
    fn from(value: vk::PhysicalDeviceProperties) -> Self {
        Self {
            api_version: value.api_version,
            driver_version: value.driver_version,
            vendor_id: value.vendor_id,
            device_id: value.device_id,
            device_type: value.device_type,
            device_name: from_i8_array(&value.device_name),
            pipeline_cache_uuid: value.pipeline_cache_uuid,
            limits: value.limits,
            sparse_properties: value.sparse_properties,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionProperties {
    pub extension_name: String,
    pub spec_version: u32,
}

impl From<vk::ExtensionProperties> for ExtensionProperties {
    fn from(value: vk::ExtensionProperties) -> Self {
        Self {
            extension_name: from_i8_array(&value.extension_name),
            spec_version: value.spec_version,
        }
    }
}
