use flutter_rust_bridge::frb;

#[frb]
pub fn set_pad_mask(pad: u8, mask: u8) -> Result<(), String> {
    crate::runtime_handle().set_pad_mask(pad as usize, mask);
    Ok(())
}

#[frb]
pub fn set_turbo_mask(pad: u8, mask: u8) -> Result<(), String> {
    crate::runtime_handle().set_turbo_mask(pad as usize, mask);
    Ok(())
}

#[frb]
pub fn set_turbo_frames_per_toggle(frames: u8) -> Result<(), String> {
    crate::runtime_handle().set_turbo_timing(frames, frames);
    Ok(())
}

#[frb]
pub fn set_turbo_timing(on_frames: u8, off_frames: u8) -> Result<(), String> {
    crate::runtime_handle().set_turbo_timing(on_frames, off_frames);
    Ok(())
}
