use nalgebra_glm::{IVec2, UVec2};

pub use sdl::video::FullscreenType;
use sdl::video::WindowBuilder;

#[derive(Default, Clone, Copy)]
pub enum WindowPosition {
    #[default]
    Undefined,
    Centered,
    Positioned(IVec2),
}

pub struct WindowCreateInfo {
    pub title: String,
    pub size: UVec2,
    pub position: WindowPosition,
    pub fullscreen_type: FullscreenType,
    pub borderless: bool,
    pub resizable: bool,
    pub vulkan_enabled: bool,
}

impl WindowCreateInfo {
    pub fn normal_window(title: String, size: UVec2, position: WindowPosition) -> Self {
        Self {
            title,
            size,
            position,
            fullscreen_type: FullscreenType::Off,
            borderless: false,
            resizable: true,
            vulkan_enabled: true,
        }
    }

    pub fn borderless_window(title: String, size: UVec2, position: WindowPosition) -> Self {
        Self {
            borderless: true,
            resizable: false,
            ..Self::normal_window(title, size, position)
        }
    }

    pub fn fullscreen_window(title: String) -> Self {
        Self {
            fullscreen_type: FullscreenType::True,
            ..Self::normal_window(title, Default::default(), Default::default())
        }
    }

    pub(crate) fn builder(&self, video: &sdl::VideoSubsystem) -> WindowBuilder {
        let WindowCreateInfo {
            title,
            size,
            position,
            fullscreen_type,
            borderless,
            vulkan_enabled,
            resizable,
            ..
        } = self;

        let mut builder = video.window(title, size.x, size.y);

        if *vulkan_enabled {
            builder.vulkan();
        }

        match fullscreen_type {
            FullscreenType::Off => {
                if *borderless {
                    builder.borderless();
                }

                if *resizable {
                    builder.resizable();
                }

                match position {
                    WindowPosition::Undefined => {}
                    WindowPosition::Centered => {
                        builder.position_centered();
                    }
                    WindowPosition::Positioned(pos) => {
                        builder.position(pos.x, pos.y);
                    }
                }
            }
            FullscreenType::True => {
                builder.fullscreen();
            }
            FullscreenType::Desktop => {
                builder.borderless().maximized();
            }
        }

        builder
    }
}
