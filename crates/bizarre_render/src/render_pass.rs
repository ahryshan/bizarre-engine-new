use std::collections::HashMap;

use ash::vk;
use bizarre_core::Handle;

use crate::device::VulkanDevice;

// TODO: make tools for generic render pass description

pub type RenderPassHandle = Handle<vk::RenderPass>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RenderPass {
    Basic,
    Custom(RenderPassHandle),
}

pub fn basic_render_pass(device: &VulkanDevice) -> Result<vk::RenderPass, vk::Result> {
    let output_attachment = vk::AttachmentDescription::default()
        .format(vk::Format::R8G8B8A8_SRGB)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
        .samples(vk::SampleCountFlags::TYPE_1);

    let output_attachment_ref = vk::AttachmentReference::default()
        .attachment(0)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

    let subpass_attachments = [output_attachment_ref];

    let subpass = vk::SubpassDescription::default()
        .color_attachments(&subpass_attachments)
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS);

    let pass_attachments = [output_attachment];
    let subpasses = [subpass];

    let create_info = vk::RenderPassCreateInfo::default()
        .attachments(&pass_attachments)
        .subpasses(&subpasses);

    unsafe { device.create_render_pass(&create_info, None) }
}
