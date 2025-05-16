use anyhow::Context;
use utils::platforms::ToUtf8String;
use windows::Foundation::TypedEventHandler;
use windows::Graphics::Capture::{Direct3D11CaptureFramePool, GraphicsCaptureItem};
use windows::Graphics::DirectX::Direct3D11::IDirect3DDevice;
use windows::Graphics::DirectX::DirectXPixelFormat;
use windows::UI::WindowId;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Dxgi::Common::{
    DXGI_FORMAT, DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_FORMAT_R16G16B16A16_FLOAT,
};
use windows::Win32::Graphics::Gdi::{MONITOR_DEFAULTTONEAREST, MonitorFromWindow};
use windows::Win32::System::Com::{COWAIT_WAITALL, CoWaitForMultipleHandles};
use windows::Win32::UI::WindowsAndMessaging::{DispatchMessageW, GetMessageW, MSG};
use windows::Win32::{
    Foundation::HMODULE,
    Graphics::{
        Direct3D::{
            D3D_DRIVER_TYPE_UNKNOWN, D3D_FEATURE_LEVEL, D3D_FEATURE_LEVEL_10_0,
            D3D_FEATURE_LEVEL_10_1, D3D_FEATURE_LEVEL_11_0,
        },
        Direct3D11::{D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_SDK_VERSION, D3D11CreateDevice},
        Dxgi::{CreateDXGIFactory1, IDXGIDevice, IDXGIFactory1},
    },
    System::WinRT::Direct3D11::CreateDirect3D11DeviceFromDXGIDevice,
};
use windows::core::Interface;

static FEATURE_LEVELS: &[D3D_FEATURE_LEVEL] = &[
    D3D_FEATURE_LEVEL_11_0,
    D3D_FEATURE_LEVEL_10_1,
    D3D_FEATURE_LEVEL_10_0,
];

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

pub fn device_create(window: HWND) -> anyhow::Result<()> {
    let factory: IDXGIFactory1 = unsafe { CreateDXGIFactory1().unwrap() };
    logs_adapter(&factory)?;
    let adapter = unsafe { factory.EnumAdapters1(0).unwrap() };
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
        )
        .unwrap()
    };
    println!("Device created successfully");
    let device = device.unwrap();
    let dxgi_device = device.cast::<IDXGIDevice>()?;
    let inspectable = unsafe { CreateDirect3D11DeviceFromDXGIDevice(&dxgi_device)? };
    let window_id = WindowId {
        Value: window.0 as u64,
    };
    let item = GraphicsCaptureItem::TryCreateFromWindowId(window_id)?;
    let device = inspectable.cast::<IDirect3DDevice>()?;
    let size = item.Size()?;
    let format = get_pixel_format(window, false);
    println!("Size: {size:?}");
    println!("Format: {format:?}");
    let frame_pool =
        Direct3D11CaptureFramePool::Create(&device, DirectXPixelFormat(format.0), 2, size)?;
    let session = frame_pool.CreateCaptureSession(&item)?;
    let handler = TypedEventHandler::new(
        |sender: windows::core::Ref<'_, Direct3D11CaptureFramePool>, args| {
            println!("Frame arrived");
            if let Some(frame) = sender.as_ref() {
                let frame = frame.TryGetNextFrame()?;
            }
            Ok(())
        },
    );
    let token = frame_pool.FrameArrived(&handler)?;
    session.StartCapture()?;
    // 事件循环
    unsafe {
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            DispatchMessageW(&msg);
        }
    }

    session.Close()?;
    frame_pool.RemoveFrameArrived(token)?;
    frame_pool.Close()?;
    Ok(())
}

fn get_pixel_format(window: HWND, force_sdr: bool) -> DXGI_FORMAT {
    let sdr_format = DXGI_FORMAT_B8G8R8A8_UNORM;
    if force_sdr {
        return sdr_format;
    }
    let monitor = unsafe { MonitorFromWindow(window, MONITOR_DEFAULTTONEAREST) };
    // TODO: 获取 monitor 是否是 HDR
    let is_hdr = false;
    if is_hdr {
        DXGI_FORMAT_R16G16B16A16_FLOAT
    } else {
        sdr_format
    }
}
