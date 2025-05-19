use windows::{
    Win32::{
        Devices::Display::{
            DISPLAYCONFIG_DEVICE_INFO_GET_SDR_WHITE_LEVEL,
            DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME, DISPLAYCONFIG_MODE_INFO,
            DISPLAYCONFIG_OUTPUT_TECHNOLOGY_DISPLAYPORT_EMBEDDED,
            DISPLAYCONFIG_OUTPUT_TECHNOLOGY_INTERNAL, DISPLAYCONFIG_OUTPUT_TECHNOLOGY_UDI_EMBEDDED,
            DISPLAYCONFIG_PATH_INFO, DISPLAYCONFIG_SDR_WHITE_LEVEL,
            DISPLAYCONFIG_SOURCE_DEVICE_NAME, DISPLAYCONFIG_VIDEO_OUTPUT_TECHNOLOGY,
            DisplayConfigGetDeviceInfo, GetDisplayConfigBufferSizes, QDC_ONLY_ACTIVE_PATHS,
            QueryDisplayConfig,
        },
        Foundation::ERROR_INSUFFICIENT_BUFFER,
        Graphics::Gdi::{GetMonitorInfoW, HMONITOR, MONITORINFOEXW},
    },
    core::{HRESULT, PCWSTR},
};

use crate::util;
use std::mem;

#[derive(Debug, Clone, Copy)]
pub struct GsMonitorColorInfo {
    pub hdr: bool,
    pub bits_per_color: u32,
    pub sdr_white_nits: u64,
}

// Returns true if this is an integrated display panel e.g. the screen attached to tablets or laptops.
fn is_internal_video_output(
    video_output_technology_type: DISPLAYCONFIG_VIDEO_OUTPUT_TECHNOLOGY,
) -> bool {
    matches!(
        video_output_technology_type,
        DISPLAYCONFIG_OUTPUT_TECHNOLOGY_INTERNAL
            | DISPLAYCONFIG_OUTPUT_TECHNOLOGY_DISPLAYPORT_EMBEDDED
            | DISPLAYCONFIG_OUTPUT_TECHNOLOGY_UDI_EMBEDDED
    )
}

fn get_device_path_info(psz_device_name: PCWSTR) -> anyhow::Result<DISPLAYCONFIG_PATH_INFO> {
    let mut num_path_array_elements = 0u32;
    let mut num_mode_info_array_elements = 0u32;
    let mut path_info_array;
    let mut mode_info_array;
    let mut hr;
    loop {
        unsafe {
            hr = GetDisplayConfigBufferSizes(
                QDC_ONLY_ACTIVE_PATHS,
                &mut num_path_array_elements,
                &mut num_mode_info_array_elements,
            );
            if !hr.is_err() {
                return Err(util::get_last_error("GetDisplayConfigBufferSizes"));
            }
            path_info_array =
                vec![DISPLAYCONFIG_PATH_INFO::default(); num_path_array_elements as usize];
            mode_info_array =
                vec![DISPLAYCONFIG_MODE_INFO::default(); num_mode_info_array_elements as usize];
            hr = QueryDisplayConfig(
                QDC_ONLY_ACTIVE_PATHS,
                &mut num_path_array_elements,
                path_info_array.as_mut_ptr(),
                &mut num_mode_info_array_elements,
                mode_info_array.as_mut_ptr(),
                None,
            );
            if hr != ERROR_INSUFFICIENT_BUFFER {
                break;
            }
        }
    }
    let mut desired_path_idx = -1i32;
    if hr.is_ok() {
        for path_idx in 0..num_mode_info_array_elements {
            let mut source_name = DISPLAYCONFIG_SOURCE_DEVICE_NAME::default();
            source_name.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME;
            source_name.header.size = size_of::<DISPLAYCONFIG_SOURCE_DEVICE_NAME>() as u32;
            source_name.header.adapterId = path_info_array[path_idx as usize].sourceInfo.adapterId;
            source_name.header.id = path_info_array[path_idx as usize].sourceInfo.id;
            let hr = HRESULT::from_nt(unsafe {
                DisplayConfigGetDeviceInfo(&mut source_name.header as *mut _)
            });
            if hr.is_ok()
                && psz_device_name == PCWSTR::from_raw(source_name.viewGdiDeviceName.as_ptr())
                && (desired_path_idx == -1
                    || is_internal_video_output(
                        path_info_array[path_idx as usize]
                            .targetInfo
                            .outputTechnology,
                    ))
            {
                desired_path_idx = path_idx as i32;
            }
        }
    }
    if desired_path_idx != -1 {
        return Ok(path_info_array[desired_path_idx as usize]);
    };
    anyhow::bail!("GetDevicePathInfo Error: E_INVALIDARG");
}

pub fn get_monitor_path_info(monitor: HMONITOR) -> anyhow::Result<DISPLAYCONFIG_PATH_INFO> {
    let mut view_info = MONITORINFOEXW::default();
    let hr = unsafe {
        GetMonitorInfoW(
            monitor,
            mem::transmute::<
                *mut windows::Win32::Graphics::Gdi::MONITORINFOEXW,
                *mut windows::Win32::Graphics::Gdi::MONITORINFO,
            >(&mut view_info as *mut _),
        )
    };
    if !hr.as_bool() {
        return Err(util::get_last_error("GetMonitorInfoW"));
    }
    let path_info = get_device_path_info(PCWSTR::from_raw(view_info.szDevice.as_ptr()))?;
    Ok(path_info)
}

pub fn get_sdr_max_nits(monitor: HMONITOR) -> anyhow::Result<u64> {
    let mut nits = 80_u64;
    let info = get_monitor_path_info(monitor)?;
    let target_info = info.targetInfo;
    let mut level = DISPLAYCONFIG_SDR_WHITE_LEVEL::default();
    level.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_SDR_WHITE_LEVEL;
    level.header.size = size_of::<DISPLAYCONFIG_SDR_WHITE_LEVEL>() as u32;
    level.header.adapterId = target_info.adapterId;
    level.header.id = target_info.id;
    let hr = HRESULT::from_nt(unsafe { DisplayConfigGetDeviceInfo(&mut level.header as *mut _) });
    if hr.is_ok() {
        nits = level.SDRWhiteLevel as u64 * 80 / 1000;
    }
    Ok(nits)
}
