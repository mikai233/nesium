/// Minimal PPU timing/debug snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NmiDebugState {
    pub nmi_output: bool,
    pub nmi_pending: bool,
    pub scanline: i16,
    pub cycle: u16,
    pub frame: u32,
}
