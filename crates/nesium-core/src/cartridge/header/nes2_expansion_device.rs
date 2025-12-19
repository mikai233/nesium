use super::Nes2DefaultExpansionDevice;

/// NES 2.0: default expansion device id (header byte 15 bits 0..=5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Nes2ExpansionDevice(pub u8);

impl Nes2ExpansionDevice {
    pub fn kind(self) -> Nes2DefaultExpansionDevice {
        Nes2DefaultExpansionDevice::from_id(self.0)
    }
}
