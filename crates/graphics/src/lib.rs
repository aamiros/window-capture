use std::sync::{Arc, Mutex};

use common::GsColorFormat;
use d3d11::GsDevice;


fn is_pow2(size: u32) -> bool {
    size >= 2 && (size & (size - 1)) == 0
}

pub struct GraphicsSubsystem {
    gs_device: Arc<Mutex<GsDevice>>,
}

impl GraphicsSubsystem {
    pub fn gs_texture_create(
        &self,
        width: u32,
        height: u32,
        color_format: GsColorFormat,
        mut levels: u32,
        mut flags: u32,
    ) -> anyhow::Result<()> {
        let pow2tex = is_pow2(width) && is_pow2(height);
        let mut uses_mipmaps = (flags & GS_BUILD_MIPMAPS) != 0 || levels != 1;

        if uses_mipmaps && !pow2tex {
            log::warn!(
                "Cannot use mipmaps with a non-power-of-two texture. Disabling mipmaps for this texture."
            );
            uses_mipmaps = false;
            flags &= !GS_BUILD_MIPMAPS;
            levels = 1;
        }

        if uses_mipmaps && (flags & GS_BUILD_MIPMAPS) != 0 {
            log::warn!(
                "Cannot use mipmaps with render targets. Disabling mipmaps for this texture."
            );
            flags &= !GS_BUILD_MIPMAPS;
            levels = 1;
        }

        Ok(())
    }
}
