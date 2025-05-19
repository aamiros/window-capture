use common::GsColorFormat;
use utils::platforms::ToUtf8String;
use windows::Win32::{
    Foundation::HMODULE,
    Graphics::{
        Direct3D::{D3D_DRIVER_TYPE_UNKNOWN, D3D_FEATURE_LEVEL_10_0},
        Direct3D11::{
            D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_SDK_VERSION, D3D11CreateDevice, ID3D11Device,
            ID3D11DeviceContext,
        },
        Dxgi::{
            Common::DXGI_COLOR_SPACE_RGB_FULL_G2084_NONE_P2020, CreateDXGIFactory1, IDXGIAdapter1,
            IDXGIFactory1, IDXGIOutput6,
        },
        Gdi::HMONITOR,
    },
};
use windows::core::Interface;

use crate::{
    FEATURE_LEVELS,
    gs_monitor_color_info::{GsMonitorColorInfo, get_sdr_max_nits},
};

pub struct GsDevice {
    pub device: ID3D11Device,
    pub factory: IDXGIFactory1,
    pub context: ID3D11DeviceContext,
    pub adapter: IDXGIAdapter1,
    pub monitor_to_hdr: Vec<(HMONITOR, GsMonitorColorInfo)>,
}

fn logs_adapter(factory: &IDXGIFactory1) -> anyhow::Result<()> {
    let mut index = 0;
    while let Ok(adapter) = unsafe { factory.EnumAdapters1(index) } {
        let desc = unsafe { adapter.GetDesc()? };
        println!("Adapter {index}:\n");
        println!("  Description: {}\n", desc.Description.to_utf8());
        println!("  Vendor ID: {}\n", desc.VendorId);
        println!("  Device ID: {}\n", desc.DeviceId);
        println!("  Sub Sys ID: {}\n", desc.SubSysId);

        index += 1;
    }
    Ok(())
}

impl GsDevice {
    pub fn init_factory() -> anyhow::Result<IDXGIFactory1> {
        let factory: IDXGIFactory1 = unsafe { CreateDXGIFactory1()? };
        Ok(factory)
    }
}

impl GsDevice {
    pub fn new(adapter_index: u32) -> anyhow::Result<Self> {
        let factory: IDXGIFactory1 = Self::init_factory()?;
        logs_adapter(&factory)?;
        let adapter = unsafe { factory.EnumAdapters1(adapter_index)? };
        let mut device = None;
        let mut level_used = D3D_FEATURE_LEVEL_10_0;
        let mut context = None;
        println!("Creating device...");
        unsafe {
            D3D11CreateDevice(
                &adapter,
                D3D_DRIVER_TYPE_UNKNOWN,
                HMODULE::default(),
                D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                Some(FEATURE_LEVELS),
                D3D11_SDK_VERSION,
                Some(&mut device),
                Some(&mut level_used),
                Some(&mut context),
            )?
        };
        let Some(device) = device else {
            return Err(anyhow::anyhow!("Failed to create device"));
        };
        let Some(context) = context else {
            return Err(anyhow::anyhow!("Failed to create context"));
        };
        Ok(Self {
            device,
            factory,
            context,
            adapter,
            monitor_to_hdr: Vec::new(),
        })
    }

    pub fn get_monitor_color_info(
        &mut self,
        h_monitor: HMONITOR,
    ) -> anyhow::Result<GsMonitorColorInfo> {
        if unsafe { !self.factory.IsCurrent() }.as_bool() {
            self.factory = Self::init_factory()?;
            self.monitor_to_hdr.clear();
        }

        for pair in &self.monitor_to_hdr {
            if pair.0 == h_monitor {
                return Ok(pair.1);
            }
        }

        let mut adapter_index = 0;

        unsafe {
            while let Ok(adapter) = self.factory.EnumAdapters1(adapter_index) {
                let mut output_index = 0;
                while let Ok(output) = adapter.EnumOutputs(output_index) {
                    let output6 = output.cast::<IDXGIOutput6>()?;
                    let desc1 = output6.GetDesc1()?;
                    if desc1.Monitor == h_monitor {
                        let hdr = desc1.ColorSpace == DXGI_COLOR_SPACE_RGB_FULL_G2084_NONE_P2020;
                        let bits = desc1.BitsPerColor;
                        let nits = get_sdr_max_nits(desc1.Monitor)?;
                        let monitor_color_info = GsMonitorColorInfo {
                            hdr,
                            bits_per_color: bits,
                            sdr_white_nits: nits,
                        };
                        self.monitor_to_hdr.push((h_monitor, monitor_color_info));
                        return Ok(monitor_color_info);
                    }
                    output_index += 1;
                }
                adapter_index += 1;
            }
        }
        Ok(GsMonitorColorInfo {
            hdr: false,
            bits_per_color: 8,
            sdr_white_nits: 80,
        })
    }

    pub fn device_is_monitor_hdr(&mut self, h_monitor: HMONITOR) -> anyhow::Result<bool> {
        Ok(self.get_monitor_color_info(h_monitor)?.hdr)
    }

    pub fn device_texture_create(
        &mut self,
        width: u32,
        height: u32,
        color_format: GsColorFormat,
        levels: u32,
        flags: u32,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}
