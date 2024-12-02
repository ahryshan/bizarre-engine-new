use ash::vk;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Antialiasing {
    None,
    FSAA,
    MSAA(MsaaFactor),
}

impl From<Antialiasing> for vk::SampleCountFlags {
    fn from(value: Antialiasing) -> Self {
        match value {
            Antialiasing::None => vk::SampleCountFlags::TYPE_1,
            Antialiasing::FSAA => vk::SampleCountFlags::TYPE_1,
            Antialiasing::MSAA(msaa_factor) => match msaa_factor {
                MsaaFactor::X2 => vk::SampleCountFlags::TYPE_2,
                MsaaFactor::X4 => vk::SampleCountFlags::TYPE_4,
                MsaaFactor::X8 => vk::SampleCountFlags::TYPE_8,
                MsaaFactor::X16 => vk::SampleCountFlags::TYPE_16,
                MsaaFactor::X32 => vk::SampleCountFlags::TYPE_32,
                MsaaFactor::X64 => vk::SampleCountFlags::TYPE_64,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MsaaFactor {
    X2,
    X4,
    X8,
    X16,
    X32,
    X64,
}
