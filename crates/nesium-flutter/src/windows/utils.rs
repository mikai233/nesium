use std::ffi::c_void;
use windows::Win32::Foundation::E_INVALIDARG;
use windows::Win32::Graphics::Direct3D::D3D11_SRV_DIMENSION_TEXTURE2D;
use windows::Win32::Graphics::Direct3D11::{
    D3D11_RENDER_TARGET_VIEW_DESC, D3D11_RTV_DIMENSION_TEXTURE2D, D3D11_SHADER_RESOURCE_VIEW_DESC,
    D3D11_TEX2D_RTV, D3D11_TEX2D_SRV, ID3D11Device, ID3D11RenderTargetView, ID3D11Resource,
    ID3D11ShaderResourceView,
};
use windows::Win32::Graphics::Dxgi::Common::{
    DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_FORMAT_R8G8B8A8_UNORM,
};
use windows::core::{HRESULT, Interface};

pub(crate) fn hresult_from_windows_error(e: &windows::core::Error) -> HRESULT {
    e.code().into()
}

pub(crate) fn log_hresult_context(prefix: &str, hr: HRESULT) {
    if hr == E_INVALIDARG {
        tracing::error!("{}: HRESULT=0x{:08X} (E_INVALIDARG)", prefix, hr.0 as u32);
    } else {
        tracing::error!("{}: HRESULT=0x{:08X}", prefix, hr.0 as u32);
    }
}

/// Validate that a resource belongs to the same device pointer we were given.
/// Mixing resources from different devices is a very common cause of E_INVALIDARG.
///
/// # Safety
/// The `res` must be a valid pointer to a D3D11 resource.
pub(crate) unsafe fn validate_resource_device(
    label: &str,
    res: &ID3D11Resource,
    expected_device_ptr: *mut c_void,
) -> Result<(), HRESULT> {
    // SAFETY: The caller must ensure 'res' is a valid interface.
    // GetDevice is a safe method on the ID3D11Resource interface.
    let owning = unsafe { res.GetDevice() };

    let Ok(owning_dev) = owning else {
        tracing::error!("{} resource has no owning device", label);
        return Err(E_INVALIDARG);
    };

    let owning_ptr = owning_dev.as_raw() as *mut c_void;
    if owning_ptr != expected_device_ptr {
        tracing::error!(
            "{} resource device mismatch: owning={:p}, expected={:p}",
            label,
            owning_ptr,
            expected_device_ptr
        );
        return Err(E_INVALIDARG);
    }

    Ok(())
}

pub(crate) fn create_srv(
    device: &ID3D11Device,
    resource: &ID3D11Resource,
) -> Result<ID3D11ShaderResourceView, HRESULT> {
    let mut srv: Option<ID3D11ShaderResourceView> = None;
    let mut srv_desc = D3D11_SHADER_RESOURCE_VIEW_DESC::default();
    srv_desc.Format = DXGI_FORMAT_R8G8B8A8_UNORM;
    srv_desc.ViewDimension = D3D11_SRV_DIMENSION_TEXTURE2D;
    srv_desc.Anonymous.Texture2D = D3D11_TEX2D_SRV {
        MipLevels: 1,
        MostDetailedMip: 0,
    };

    // SAFETY: We wrap the FFI call to CreateShaderResourceView.
    // The device and resource objects are valid references.
    unsafe {
        if let Err(e) = device.CreateShaderResourceView(resource, Some(&srv_desc), Some(&mut srv)) {
            let hr = hresult_from_windows_error(&e);
            if hr == E_INVALIDARG {
                tracing::warn!("Retry CreateShaderResourceView with inferred desc");
                let mut srv_fallback = None;
                if let Err(e2) =
                    device.CreateShaderResourceView(resource, None, Some(&mut srv_fallback))
                {
                    return Err(hresult_from_windows_error(&e2));
                }
                return srv_fallback.ok_or(E_INVALIDARG);
            }
            return Err(hr);
        }
    }

    srv.ok_or(E_INVALIDARG)
}

pub(crate) fn create_rtv(
    device: &ID3D11Device,
    resource: &ID3D11Resource,
) -> Result<ID3D11RenderTargetView, HRESULT> {
    let mut rtv: Option<ID3D11RenderTargetView> = None;
    let mut rtv_desc = D3D11_RENDER_TARGET_VIEW_DESC::default();
    rtv_desc.Format = DXGI_FORMAT_B8G8R8A8_UNORM;
    rtv_desc.ViewDimension = D3D11_RTV_DIMENSION_TEXTURE2D;
    rtv_desc.Anonymous.Texture2D = D3D11_TEX2D_RTV { MipSlice: 0 };

    // SAFETY: We wrap the FFI call to CreateRenderTargetView.
    // The device and resource objects are valid references.
    unsafe {
        if let Err(e) = device.CreateRenderTargetView(resource, Some(&rtv_desc), Some(&mut rtv)) {
            let hr = hresult_from_windows_error(&e);
            if hr == E_INVALIDARG {
                tracing::warn!("Retry CreateRenderTargetView with inferred desc");
                let mut rtv_fallback = None;
                if let Err(e2) =
                    device.CreateRenderTargetView(resource, None, Some(&mut rtv_fallback))
                {
                    return Err(hresult_from_windows_error(&e2));
                }
                return rtv_fallback.ok_or(E_INVALIDARG);
            }
            return Err(hr);
        }
    }

    rtv.ok_or(E_INVALIDARG)
}
