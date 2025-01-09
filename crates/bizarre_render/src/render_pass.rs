use std::collections::HashMap;

use ash::vk;
use bizarre_core::Handle;

use crate::device::LogicalDevice;

// TODO: make tools for generic render pass description

pub type RenderPassHandle = Handle<VulkanRenderPass>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RenderPassAttachment {
    Resolve,
    Output(vk::SampleCountFlags),
    Color(vk::SampleCountFlags),
    DepthStencil(vk::SampleCountFlags),
}

pub struct VulkanRenderPass {
    pub render_pass: vk::RenderPass,
    pub attachments: Box<[RenderPassAttachment]>,
    pub samples: vk::SampleCountFlags,
}

impl VulkanRenderPass {
    pub fn msaa(&self) -> bool {
        self.samples != vk::SampleCountFlags::TYPE_1
    }
}

pub fn deferred_render_pass(
    device: &LogicalDevice,
    samples: vk::SampleCountFlags,
) -> Result<VulkanRenderPass, vk::Result> {
    let msaa = samples != vk::SampleCountFlags::TYPE_1;

    let color = vk::AttachmentDescription2::default()
        .format(vk::Format::R8G8B8A8_SRGB)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        .samples(samples);

    let normals = vk::AttachmentDescription2::default()
        .format(vk::Format::R8G8B8A8_SRGB)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        .samples(samples);

    let depth = vk::AttachmentDescription2::default()
        .format(vk::Format::D32_SFLOAT)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
        .samples(samples);

    let output = vk::AttachmentDescription2::default()
        .format(vk::Format::R8G8B8A8_SRGB)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(if msaa {
            vk::AttachmentStoreOp::DONT_CARE
        } else {
            vk::AttachmentStoreOp::STORE
        })
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(if msaa {
            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL
        } else {
            vk::ImageLayout::TRANSFER_SRC_OPTIMAL
        })
        .samples(samples);

    let mut attachments = vec![color, normals, depth, output];

    if msaa {
        let resolve = vk::AttachmentDescription2::default()
            .format(vk::Format::R8G8B8A8_SRGB)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
            .samples(vk::SampleCountFlags::TYPE_1);

        attachments.push(resolve)
    }

    let deferred_attachments = [
        vk::AttachmentReference2 {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            aspect_mask: vk::ImageAspectFlags::COLOR,
            ..Default::default()
        },
        vk::AttachmentReference2 {
            attachment: 1,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            aspect_mask: vk::ImageAspectFlags::COLOR,
            ..Default::default()
        },
    ];

    let depth_ref = vk::AttachmentReference2 {
        attachment: 2,
        layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        aspect_mask: vk::ImageAspectFlags::DEPTH,
        ..Default::default()
    };

    let deferred_subpass = vk::SubpassDescription2::default()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(&deferred_attachments)
        .depth_stencil_attachment(&depth_ref);

    let composition_attachments = [vk::AttachmentReference2 {
        attachment: 3,
        layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        aspect_mask: vk::ImageAspectFlags::COLOR,
        ..Default::default()
    }];

    let composition_input = [
        vk::AttachmentReference2 {
            attachment: 0,
            layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            aspect_mask: vk::ImageAspectFlags::COLOR,
            ..Default::default()
        },
        vk::AttachmentReference2 {
            attachment: 1,
            layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            aspect_mask: vk::ImageAspectFlags::COLOR,
            ..Default::default()
        },
    ];

    let composition_subpass = vk::SubpassDescription2::default()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(&composition_attachments)
        .input_attachments(&composition_input);

    let resolve_color = [vk::AttachmentReference2 {
        attachment: 3,
        layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        aspect_mask: vk::ImageAspectFlags::COLOR,
        ..Default::default()
    }];

    let resolve_resolve = [vk::AttachmentReference2 {
        attachment: 4,
        layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        aspect_mask: vk::ImageAspectFlags::COLOR,
        ..Default::default()
    }];

    let resolve_subpass = vk::SubpassDescription2::default()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(&resolve_color)
        .resolve_attachments(&resolve_resolve);

    let mut subpasses = vec![deferred_subpass, composition_subpass];

    if msaa {
        subpasses.push(resolve_subpass)
    }

    let dependencies = [
        vk::SubpassDependency2 {
            src_subpass: vk::SUBPASS_EXTERNAL,
            dst_subpass: 0,
            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            ..Default::default()
        },
        vk::SubpassDependency2 {
            src_subpass: 0,
            dst_subpass: 1,
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            src_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            dst_stage_mask: vk::PipelineStageFlags::FRAGMENT_SHADER,
            dst_access_mask: vk::AccessFlags::INPUT_ATTACHMENT_READ,
            ..Default::default()
        },
    ];

    let create_info = vk::RenderPassCreateInfo2::default()
        .attachments(&attachments)
        .dependencies(&dependencies)
        .subpasses(&subpasses);

    let handle = unsafe { device.create_render_pass2(&create_info, None)? };

    let mut attachments = vec![
        RenderPassAttachment::Color(samples),
        RenderPassAttachment::Color(samples),
        RenderPassAttachment::DepthStencil(samples),
        RenderPassAttachment::Output(samples),
    ];

    if msaa {
        attachments.push(RenderPassAttachment::Resolve);
    }

    let attachments = attachments.into_boxed_slice();

    Ok(VulkanRenderPass {
        render_pass: handle,
        attachments,
        samples,
    })
}
