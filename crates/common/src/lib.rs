#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum GsColorFormat {
    GsUnknown,
    GsA8,
    GsR8,
    GsRGBA,
    GsBGRX,
    GsBGRA,
    GsR10G10B10A2,
    GsRGBA16,
    GsR16,
    GsRGBA16F,
    GsRGBA32F,
    GsRG16F,
    GsRG32F,
    GsR16F,
    GsR32F,
    GsDXT1,
    GsDXT3,
    GsDXT5,
    GsR8G8,
    GsRgbaUnorm,
    GsBgrxUnorm,
    GsBgraUnorm,
    GsRg16,
}

pub const GS_BUILD_MIPMAPS: u32 = 1 << 0;
pub const GS_DYNAMIC: u32 = 1 << 1;
pub const GS_RENDER_TARGET: u32 = 1 << 2;
pub const GS_GL_DUMMY_TEXTURE: u32 = 1 << 3;
