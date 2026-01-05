//! Common capture point logic for debug viewers.

/// Defines when to capture a snapshot from an interceptor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CapturePoint {
    #[default]
    Disabled,
    FrameStart,
    VblankStart,
    ScanlineDot {
        scanline: i16,
        dot: u16,
    },
}

impl CapturePoint {
    #[inline]
    pub fn should_capture_on_frame_start(self) -> bool {
        matches!(self, Self::FrameStart)
    }

    #[inline]
    pub fn should_capture_on_vblank_start(self) -> bool {
        matches!(self, Self::VblankStart)
    }

    #[inline]
    pub fn should_capture_on_scanline_dot(self, scanline: i16, dot: u16) -> bool {
        matches!(
            self,
            Self::ScanlineDot {
                scanline: target_scanline,
                dot: target_dot,
            } if scanline == target_scanline && dot == target_dot
        )
    }
}
