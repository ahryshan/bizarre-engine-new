use ash::vk;
use bitflags::bitflags;

#[derive(Clone, Default, Debug)]
pub struct VulkanPipelineFeatures {
    pub flags: PipelineFeatureFlags,
    pub culling: CullMode,
    pub primitive_topology: PrimitiveTopology,
    pub polygon_mode: PolygonMode,
}

bitflags! {
    #[repr(transparent)]
    #[derive(Copy, Clone, Default, Debug, PartialEq)]
    pub struct PipelineFeatureFlags: usize {
        const BLEND_SHIFT = 0;
        const BLEND_FIELD_WIDTH = 4;
        const BLEND_MASK = 0xf << Self::BLEND_SHIFT.bits();
        const BLEND_ALPHA = 0b0001;
        const BLEND_COLOR = 0b0010;
        const BLEND_COLOR_ALPHA = 0b0011;
        const BLEND_ADD = 0b0100;

        const DEPTH_SHIFT = Self::BLEND_SHIFT.bits() + Self::BLEND_FIELD_WIDTH.bits();
        const DEPTH_FIELD_WIDTH = 4;
        const DEPTH_MASK = 0xf << Self::DEPTH_SHIFT.bits();
        const DEPTH_TEST = 0b0001 << Self::DEPTH_SHIFT.bits();
        const DEPTH_WRITE = 0b0010 << Self::DEPTH_SHIFT.bits();

        const STENCIL_SHIFT = Self::DEPTH_SHIFT.bits() + Self::DEPTH_FIELD_WIDTH.bits();
        const STENCIL_FIELD_WIDTH = 4;
        const STENCIL_MASK = 0xf << Self::STENCIL_SHIFT.bits();
        const STENCIL_TEST = 0b0001 << Self::STENCIL_SHIFT.bits();
    }
}

#[repr(u32)]
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CullMode {
    #[default]
    None = 0,
    Front = 0b01,
    Back = 0b10,
    FrontAndBack = 0b11,
}

impl From<CullMode> for vk::CullModeFlags {
    fn from(value: CullMode) -> Self {
        vk::CullModeFlags::from_raw(value as u32)
    }
}

impl From<vk::CullModeFlags> for CullMode {
    fn from(value: vk::CullModeFlags) -> Self {
        unsafe { std::mem::transmute_copy(&value) }
    }
}

#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub enum PrimitiveTopology {
    PointList,
    LineList,
    LineStrip,
    #[default]
    TriangleList,
    TriangleStrip,
    TriangleFan,
    LineListWithAdjacency,
    LineStripWithAdjacency,
    TriangleListWithAdjacency,
    TriangleStripWithAdjacency,
    PatchList,
}

impl From<PrimitiveTopology> for vk::PrimitiveTopology {
    fn from(value: PrimitiveTopology) -> Self {
        Self::from_raw(value as i32)
    }
}

impl From<vk::PrimitiveTopology> for PrimitiveTopology {
    fn from(value: vk::PrimitiveTopology) -> Self {
        unsafe { std::mem::transmute_copy(&value) }
    }
}

#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub enum PolygonMode {
    #[default]
    Fill = 0,
    Line = 1,
    Point = 2,
}

impl From<PolygonMode> for vk::PolygonMode {
    fn from(value: PolygonMode) -> Self {
        Self::from_raw(value as i32)
    }
}
