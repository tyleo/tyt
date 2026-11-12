use crate::{Error, Result, XrGpu};
use ash::{
    khr::pipeline_executable_properties,
    vk::{
        self, AttachmentDescription, AttachmentLoadOp, AttachmentReference, AttachmentStoreOp,
        ColorComponentFlags, CompareOp, CullModeFlags, DescriptorSetLayoutBinding,
        DescriptorSetLayoutCreateInfo, DescriptorType, DynamicState, Format, FrontFace,
        GraphicsPipelineCreateInfo, ImageLayout, PipelineBindPoint, PipelineCache,
        PipelineColorBlendAttachmentState, PipelineColorBlendStateCreateInfo, PipelineCreateFlags,
        PipelineDepthStencilStateCreateInfo, PipelineDynamicStateCreateInfo,
        PipelineExecutableInfoKHR, PipelineExecutableStatisticFormatKHR, PipelineInfoKHR,
        PipelineInputAssemblyStateCreateInfo, PipelineLayoutCreateInfo,
        PipelineMultisampleStateCreateInfo, PipelineRasterizationStateCreateInfo,
        PipelineShaderStageCreateInfo, PipelineVertexInputStateCreateInfo,
        PipelineViewportStateCreateInfo, PrimitiveTopology, RenderPassCreateInfo,
        RenderPassMultiviewCreateInfo, SampleCountFlags, ShaderModuleCreateInfo, ShaderStageFlags,
        SubpassDescription, VertexInputAttributeDescription, VertexInputBindingDescription,
        VertexInputRate,
    },
};
use std::ffi::CString;
use voxrender_bench::box_view_shader;
use wgpu::{
    hal::api::Vulkan,
    naga::{
        ResourceBinding, ShaderStage,
        back::spv::{
            self, BindingInfo, Options, PipelineOptions, WriterFlags,
            ZeroInitializeWorkgroupMemoryMode,
        },
        front::wgsl,
        proc::{BoundsCheckPolicies, BoundsCheckPolicy},
        valid::{Capabilities, ValidationFlags, Validator},
    },
};

/// The view's pipelines as a vertex entry, a fragment entry, and whether
/// the pipeline draws back faces.
const PIPELINES: [(&str, &str, bool); 5] = [
    ("place_front", "front", false),
    ("place_front", "crossed", false),
    ("place_front", "solid", false),
    ("place_near", "inside", true),
    ("place_held", "held", true),
];

/// The buffers `box_view_shader` binds in group 0. Its texture and its
/// sampler follow them.
const BUFFERS: u32 = 8;

/// Logs the driver's statistics for each pipeline of the headset view's
/// shader: on Adreno, a stage's instructions, its registers, and the share
/// of its fibers the driver lets run at once.
///
/// The probe compiles each stage as wgpu compiles a trusted module and
/// builds its pipelines beside wgpu's, over a pass of the view's shape: two
/// eyes of color and 32-bit float depth. The vertex buffers follow
/// `BoxDraw`'s.
pub fn probe_pipelines(gpu: &XrGpu) -> Result<()> {
    if !gpu.statistics {
        log::info!("b0 pipeline statistics: the device reports none");

        return Ok(());
    }

    // The probe reads the shader whose mirror rays see the sky alone, which
    // binds no acceleration structure.
    let module = wgsl::parse_str(&box_view_shader(false, None, None))
        .map_err(|error| Error::ProbeShader(error.to_string()))?;

    let info = Validator::new(ValidationFlags::all(), Capabilities::all())
        .validate(&module)
        .map_err(|error| Error::ProbeShader(error.to_string()))?;

    // SAFETY: the probe makes no Vulkan call that changes wgpu's state.
    let hal_device =
        unsafe { gpu.device.as_hal::<Vulkan>() }.expect("XrGpu opens its device on Vulkan");

    let instance = hal_device.shared_instance().raw_instance();
    let device = hal_device.raw_device();

    // SAFETY: the physical device belongs to `instance`.
    let properties =
        unsafe { instance.get_physical_device_properties(hal_device.raw_physical_device()) };

    // What wgpu's Vulkan backend sets for a module with no runtime checks.
    let options = Options {
        lang_version: match vk::api_version_minor(properties.api_version) {
            0 => (1, 0),
            1 => (1, 3),
            2 => (1, 5),
            _ => (1, 6),
        },
        flags: WriterFlags::FORCE_POINT_SIZE,
        binding_map: (0..BUFFERS + 2)
            .map(|binding| {
                (
                    ResourceBinding { group: 0, binding },
                    BindingInfo {
                        descriptor_set: 0,
                        binding,
                        binding_array_size: None,
                    },
                )
            })
            .collect(),
        capabilities: None,
        bounds_check_policies: BoundsCheckPolicies {
            index: BoundsCheckPolicy::Unchecked,
            buffer: BoundsCheckPolicy::Unchecked,
            image_load: BoundsCheckPolicy::Unchecked,
            binding_array: BoundsCheckPolicy::Unchecked,
        },
        zero_initialize_workgroup_memory: ZeroInitializeWorkgroupMemoryMode::None,
        force_loop_bounding: false,
        ray_query_initialization_tracking: false,
        emit_int_div_checks: false,
        ..Options::default()
    };

    let compile = |shader_stage, entry: &str| {
        let words = spv::write_vec(
            &module,
            &info,
            &options,
            Some(&PipelineOptions {
                shader_stage,
                entry_point: entry.to_owned(),
            }),
        )
        .map_err(|error| Error::ProbeShader(error.to_string()))?;

        // SAFETY: `words` holds the SPIR-V naga wrote and outlives the call.
        unsafe {
            device.create_shader_module(&ShaderModuleCreateInfo::default().code(&words), None)
        }
        .map_err(Error::Vulkan)
    };

    let bindings: Vec<_> = (0..BUFFERS + 2)
        .map(|binding| {
            DescriptorSetLayoutBinding::default()
                .binding(binding)
                .descriptor_type(match binding {
                    0 => DescriptorType::UNIFORM_BUFFER,
                    BUFFERS => DescriptorType::SAMPLED_IMAGE,
                    binding if binding > BUFFERS => DescriptorType::SAMPLER,
                    _ => DescriptorType::STORAGE_BUFFER,
                })
                .descriptor_count(1)
                .stage_flags(ShaderStageFlags::VERTEX | ShaderStageFlags::FRAGMENT)
        })
        .collect();

    // SAFETY: each create info and what it points to are alive for its
    // call, and the probe destroys what it creates before it returns.
    let (set_layout, layout, render_pass) = unsafe {
        let set_layout = device
            .create_descriptor_set_layout(
                &DescriptorSetLayoutCreateInfo::default().bindings(&bindings),
                None,
            )
            .map_err(Error::Vulkan)?;

        let layout = device
            .create_pipeline_layout(
                &PipelineLayoutCreateInfo::default().set_layouts(&[set_layout]),
                None,
            )
            .map_err(Error::Vulkan)?;

        let attachments = [
            AttachmentDescription::default()
                .format(Format::R8G8B8A8_SRGB)
                .samples(SampleCountFlags::TYPE_1)
                .load_op(AttachmentLoadOp::CLEAR)
                .store_op(AttachmentStoreOp::STORE)
                .final_layout(ImageLayout::COLOR_ATTACHMENT_OPTIMAL),
            AttachmentDescription::default()
                .format(Format::D32_SFLOAT)
                .samples(SampleCountFlags::TYPE_1)
                .load_op(AttachmentLoadOp::CLEAR)
                .store_op(AttachmentStoreOp::DONT_CARE)
                .final_layout(ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL),
        ];

        let colors = [AttachmentReference::default()
            .attachment(0)
            .layout(ImageLayout::COLOR_ATTACHMENT_OPTIMAL)];

        let depth = AttachmentReference::default()
            .attachment(1)
            .layout(ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

        let subpasses = [SubpassDescription::default()
            .pipeline_bind_point(PipelineBindPoint::GRAPHICS)
            .color_attachments(&colors)
            .depth_stencil_attachment(&depth)];

        let both_eyes = [0b11];

        let mut multiview = RenderPassMultiviewCreateInfo::default()
            .view_masks(&both_eyes)
            .correlation_masks(&both_eyes);

        let render_pass = device
            .create_render_pass(
                &RenderPassCreateInfo::default()
                    .attachments(&attachments)
                    .subpasses(&subpasses)
                    .push_next(&mut multiview),
                None,
            )
            .map_err(Error::Vulkan)?;

        (set_layout, layout, render_pass)
    };

    let statistics = pipeline_executable_properties::Device::new(instance, device);

    for (vertex, fragment, backs) in PIPELINES {
        let vertex_module = compile(ShaderStage::Vertex, vertex)?;
        let fragment_module = compile(ShaderStage::Fragment, fragment)?;
        let vertex_name = CString::new(vertex).expect("an entry's name holds no nul");
        let fragment_name = CString::new(fragment).expect("an entry's name holds no nul");

        let stages = [
            PipelineShaderStageCreateInfo::default()
                .stage(ShaderStageFlags::VERTEX)
                .module(vertex_module)
                .name(&vertex_name),
            PipelineShaderStageCreateInfo::default()
                .stage(ShaderStageFlags::FRAGMENT)
                .module(fragment_module)
                .name(&fragment_name),
        ];

        let buffers = [
            VertexInputBindingDescription::default()
                .binding(0)
                .stride(4)
                .input_rate(VertexInputRate::VERTEX),
            VertexInputBindingDescription::default()
                .binding(1)
                .stride(48)
                .input_rate(VertexInputRate::INSTANCE),
        ];

        let attributes = [
            (0, 0, Format::R32_UINT, 0),
            (1, 1, Format::R32G32B32A32_UINT, 0),
            (2, 1, Format::R32G32B32A32_UINT, 16),
            (3, 1, Format::R32G32B32A32_UINT, 32),
        ]
        .map(|(location, binding, format, offset)| {
            VertexInputAttributeDescription::default()
                .location(location)
                .binding(binding)
                .format(format)
                .offset(offset)
        });

        let vertex_input = PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(&buffers)
            .vertex_attribute_descriptions(&attributes);

        let assembly = PipelineInputAssemblyStateCreateInfo::default()
            .topology(PrimitiveTopology::TRIANGLE_LIST);

        let viewport = PipelineViewportStateCreateInfo::default()
            .viewport_count(1)
            .scissor_count(1);

        let raster = PipelineRasterizationStateCreateInfo::default()
            .cull_mode(if backs {
                CullModeFlags::FRONT
            } else {
                CullModeFlags::BACK
            })
            .front_face(FrontFace::COUNTER_CLOCKWISE)
            .line_width(1.0);

        let multisample = PipelineMultisampleStateCreateInfo::default()
            .rasterization_samples(SampleCountFlags::TYPE_1);

        let depth = PipelineDepthStencilStateCreateInfo::default()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(CompareOp::GREATER);

        let targets = [PipelineColorBlendAttachmentState::default()
            .color_write_mask(ColorComponentFlags::RGBA)];

        let blend = PipelineColorBlendStateCreateInfo::default().attachments(&targets);
        let dynamic_states = [DynamicState::VIEWPORT, DynamicState::SCISSOR];
        let dynamic = PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

        let pipeline_info = GraphicsPipelineCreateInfo::default()
            .flags(PipelineCreateFlags::CAPTURE_STATISTICS_KHR)
            .stages(&stages)
            .vertex_input_state(&vertex_input)
            .input_assembly_state(&assembly)
            .viewport_state(&viewport)
            .rasterization_state(&raster)
            .multisample_state(&multisample)
            .depth_stencil_state(&depth)
            .color_blend_state(&blend)
            .dynamic_state(&dynamic)
            .layout(layout)
            .render_pass(render_pass);

        // SAFETY: `pipeline_info` and what it points to are alive for the
        // call. The device has the statistics' feature, and the pipeline
        // was created to capture them. The probe destroys the pipeline and
        // its modules once it has read them.
        unsafe {
            let pipeline = device
                .create_graphics_pipelines(PipelineCache::null(), &[pipeline_info], None)
                .map_err(|(_, result)| Error::Vulkan(result))?[0];

            let executables = statistics
                .get_pipeline_executable_properties(&PipelineInfoKHR::default().pipeline(pipeline))
                .map_err(Error::Vulkan)?;

            for (index, executable) in (0..).zip(&executables) {
                let values = statistics
                    .get_pipeline_executable_statistics(
                        &PipelineExecutableInfoKHR::default()
                            .pipeline(pipeline)
                            .executable_index(index),
                    )
                    .map_err(Error::Vulkan)?;

                let values: Vec<_> = values
                    .iter()
                    .map(|statistic| {
                        let name = statistic
                            .name_as_c_str()
                            .expect("a statistic's name ends in nul")
                            .to_string_lossy();

                        let value = match statistic.format {
                            PipelineExecutableStatisticFormatKHR::BOOL32 => {
                                (statistic.value.b32 != 0).to_string()
                            }

                            PipelineExecutableStatisticFormatKHR::INT64 => {
                                statistic.value.i64.to_string()
                            }

                            PipelineExecutableStatisticFormatKHR::UINT64 => {
                                statistic.value.u64.to_string()
                            }

                            _ => statistic.value.f64.to_string(),
                        };

                        format!("{name} {value}")
                    })
                    .collect();

                log::info!(
                    "b0 pipeline statistics, {vertex} and {fragment}, {}: {}",
                    executable
                        .name_as_c_str()
                        .expect("an executable's name ends in nul")
                        .to_string_lossy(),
                    values.join(", ")
                );
            }

            device.destroy_pipeline(pipeline, None);
            device.destroy_shader_module(vertex_module, None);
            device.destroy_shader_module(fragment_module, None);
        }
    }

    // SAFETY: the probe created the three, and its pipelines are gone.
    unsafe {
        device.destroy_render_pass(render_pass, None);
        device.destroy_pipeline_layout(layout, None);
        device.destroy_descriptor_set_layout(set_layout, None);
    }

    Ok(())
}
