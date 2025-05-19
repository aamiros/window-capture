use common::{GS_BUILD_MIPMAPS, GS_DYNAMIC, GS_RENDER_TARGET, GsColorFormat};
use windows::Win32::Graphics::{
    Direct3D11::{
        D3D11_BIND_RENDER_TARGET, D3D11_BIND_SHADER_RESOURCE, D3D11_CPU_ACCESS_WRITE,
        D3D11_RESOURCE_MISC_GDI_COMPATIBLE, D3D11_RESOURCE_MISC_TEXTURECUBE, D3D11_TEXTURE2D_DESC,
        D3D11_USAGE_DEFAULT, D3D11_USAGE_DYNAMIC, ID3D11Texture2D,
    },
    Dxgi::Common::{
        DXGI_FORMAT, DXGI_FORMAT_A8_UNORM, DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_FORMAT_B8G8R8X8_UNORM,
        DXGI_FORMAT_BC1_UNORM, DXGI_FORMAT_BC2_UNORM, DXGI_FORMAT_BC3_UNORM, DXGI_FORMAT_NV12,
        DXGI_FORMAT_P010, DXGI_FORMAT_R8_UNORM, DXGI_FORMAT_R8G8_UNORM, DXGI_FORMAT_R8G8B8A8_UNORM,
        DXGI_FORMAT_R10G10B10A2_UNORM, DXGI_FORMAT_R16_FLOAT, DXGI_FORMAT_R16_UNORM,
        DXGI_FORMAT_R16G16_FLOAT, DXGI_FORMAT_R16G16_UNORM, DXGI_FORMAT_R16G16B16A16_FLOAT,
        DXGI_FORMAT_R16G16B16A16_UNORM, DXGI_FORMAT_R32_FLOAT, DXGI_FORMAT_R32G32_FLOAT,
        DXGI_FORMAT_R32G32B32A32_FLOAT, DXGI_FORMAT_UNKNOWN,
    },
};

use crate::GsDevice;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GsTextureType {
    Texture2D,
    Texture3D,
    TextureCube,
}

fn convert_gs_texture_format_resource(format: GsColorFormat) -> DXGI_FORMAT {
    match format {
        GsColorFormat::GsUnknown => DXGI_FORMAT_UNKNOWN,
        GsColorFormat::GsA8 => DXGI_FORMAT_A8_UNORM,
        GsColorFormat::GsR8 => DXGI_FORMAT_R8_UNORM,
        GsColorFormat::GsRGBA => DXGI_FORMAT_R8G8B8A8_UNORM,
        GsColorFormat::GsBGRX => DXGI_FORMAT_B8G8R8X8_UNORM,
        GsColorFormat::GsBGRA => DXGI_FORMAT_B8G8R8A8_UNORM,
        GsColorFormat::GsR10G10B10A2 => DXGI_FORMAT_R10G10B10A2_UNORM,
        GsColorFormat::GsRGBA16 => DXGI_FORMAT_R16G16B16A16_UNORM,
        GsColorFormat::GsR16 => DXGI_FORMAT_R16_UNORM,
        GsColorFormat::GsRGBA16F => DXGI_FORMAT_R16G16B16A16_FLOAT,
        GsColorFormat::GsRGBA32F => DXGI_FORMAT_R32G32B32A32_FLOAT,
        GsColorFormat::GsRG16F => DXGI_FORMAT_R16G16_FLOAT,
        GsColorFormat::GsRG32F => DXGI_FORMAT_R32G32_FLOAT,
        GsColorFormat::GsR16F => DXGI_FORMAT_R16_FLOAT,
        GsColorFormat::GsR32F => DXGI_FORMAT_R32_FLOAT,
        GsColorFormat::GsDXT1 => DXGI_FORMAT_BC1_UNORM,
        GsColorFormat::GsDXT3 => DXGI_FORMAT_BC2_UNORM,
        GsColorFormat::GsDXT5 => DXGI_FORMAT_BC3_UNORM,
        GsColorFormat::GsR8G8 => DXGI_FORMAT_R8G8_UNORM,
        GsColorFormat::GsRgbaUnorm => DXGI_FORMAT_R8G8B8A8_UNORM,
        GsColorFormat::GsBgrxUnorm => DXGI_FORMAT_B8G8R8X8_UNORM,
        GsColorFormat::GsBgraUnorm => DXGI_FORMAT_B8G8R8A8_UNORM,
        GsColorFormat::GsRg16 => DXGI_FORMAT_R16G16_UNORM,
    }
}

pub struct GsTexture2D {
    texture: ID3D11Texture2D,
    width: u32,
    height: u32,
    gs_color_format: GsColorFormat,
    levels: u32,
    flags: u32,
    r#type: GsTextureType,
    gdi_compatible: bool,
    two_plane: bool,
}

pub struct GsTexture2DBuilder<'a> {
    device: &'a GsDevice,
    width: u32,
    height: u32,
    gs_color_format: GsColorFormat,
    levels: u32,
    flags: u32,
    r#type: GsTextureType,
    dxgi_format_resource: DXGI_FORMAT,
    gdi_compatible: Option<bool>,
    two_plane: Option<bool>,
    gen_mipmaps: bool,
    is_dynamic: bool,
    is_render_target: bool,
}

impl<'a> GsTexture2DBuilder<'a> {
    pub fn new(
        device: &'a GsDevice,
        width: u32,
        height: u32,
        gs_color_format: GsColorFormat,
        levels: u32,
        flags: u32,
        r#type: GsTextureType,
    ) -> Self {
        Self {
            device,
            width,
            height,
            gs_color_format,
            levels,
            flags,
            r#type,
            dxgi_format_resource: convert_gs_texture_format_resource(gs_color_format),
            gdi_compatible: None,
            two_plane: None,
            gen_mipmaps: (flags & GS_BUILD_MIPMAPS) != 0,
            is_dynamic: (flags & GS_DYNAMIC) != 0,
            is_render_target: (flags & GS_RENDER_TARGET) != 0,
        }
    }

    pub fn gdi_compatible(mut self, gdi_compatible: bool) -> Self {
        self.gdi_compatible = Some(gdi_compatible);
        self
    }

    pub fn two_plane(mut self, two_plane: bool) -> Self {
        self.two_plane = Some(two_plane);
        self
    }

    pub fn init_texture(&self) -> anyhow::Result<ID3D11Texture2D> {
        let mut td = D3D11_TEXTURE2D_DESC::default();
        td.Width = self.width;
        td.Height = self.height;
        td.MipLevels = self.levels;
        td.ArraySize = if self.r#type == GsTextureType::TextureCube {
            6
        } else {
            1
        };
        td.Format = if self.two_plane.unwrap_or(false) {
            if self.gs_color_format == GsColorFormat::GsR16 {
                DXGI_FORMAT_P010
            } else {
                DXGI_FORMAT_NV12
            }
        } else {
            self.dxgi_format_resource
        };
        td.BindFlags = D3D11_BIND_SHADER_RESOURCE.0 as u32;
        td.SampleDesc.Count = 1;
        td.CPUAccessFlags = if self.is_dynamic {
            D3D11_CPU_ACCESS_WRITE.0 as u32
        } else {
            0
        };
        td.Usage = if self.is_dynamic {
            D3D11_USAGE_DYNAMIC
        } else {
            D3D11_USAGE_DEFAULT
        };
        if self.r#type == GsTextureType::TextureCube {
            td.MiscFlags |= D3D11_RESOURCE_MISC_TEXTURECUBE.0 as u32;
        }
        let is_gdi_compatible = self.gdi_compatible.unwrap_or(false);
        if self.is_render_target || is_gdi_compatible {
            td.BindFlags |= D3D11_BIND_RENDER_TARGET.0 as u32;
        }
        if is_gdi_compatible {
            td.MiscFlags |= D3D11_RESOURCE_MISC_GDI_COMPATIBLE.0 as u32;
        }
        let mut texture = None;
        unsafe {
            self.device
                .device
                .CreateTexture2D(&td as *const _, None, Some(&mut texture))?;
        }
        Ok(texture.unwrap())
    }

    pub fn build(self) -> anyhow::Result<GsTexture2D> {
        Ok(GsTexture2D {
            width: self.width,
            height: self.height,
            gs_color_format: self.gs_color_format,
            levels: self.levels,
            flags: self.flags,
            r#type: self.r#type,
            gdi_compatible: self.gdi_compatible.unwrap_or(false),
            two_plane: self.two_plane.unwrap_or(false),
            texture: self.init_texture()?,
        })
    }
}
