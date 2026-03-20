use crate::cartridge::header::Mirroring;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NamcoVariant {
    Namco163,
    Namco175,
    Namco340,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Namco19VariantInit {
    pub variant: NamcoVariant,
    pub auto_detect_variant: bool,
    pub has_audio_block: bool,
}

pub fn namco19_variant_init(mapper_id: u16, submapper: u8) -> Namco19VariantInit {
    let (variant, auto_detect_variant) = match (mapper_id, submapper) {
        (19, _) => (NamcoVariant::Namco163, false),
        (210, 1) => (NamcoVariant::Namco175, false),
        (210, 2) => (NamcoVariant::Namco340, false),
        (210, 0) => (NamcoVariant::Unknown, true),
        _ => (NamcoVariant::Namco163, true),
    };

    Namco19VariantInit {
        variant,
        auto_detect_variant,
        has_audio_block: submapper != 2,
    }
}

#[derive(Debug, Clone)]
pub struct Namco19BoardState {
    reset_variant: NamcoVariant,
    variant: NamcoVariant,
    not_namco340: bool,
    auto_detect_variant: bool,
    write_protect: u8,
    low_chr_nt_mode: bool,
    high_chr_nt_mode: bool,
    reset_mirroring: Mirroring,
    mirroring: Mirroring,
}

impl Namco19BoardState {
    pub fn new(init: Namco19VariantInit, mirroring: Mirroring) -> Self {
        Self {
            reset_variant: init.variant,
            variant: init.variant,
            not_namco340: false,
            auto_detect_variant: init.auto_detect_variant,
            write_protect: 0,
            low_chr_nt_mode: false,
            high_chr_nt_mode: false,
            reset_mirroring: mirroring,
            mirroring,
        }
    }

    pub fn reset(&mut self) {
        self.variant = self.reset_variant;
        self.not_namco340 = false;
        self.write_protect = 0;
        self.low_chr_nt_mode = false;
        self.high_chr_nt_mode = false;
        self.mirroring = self.reset_mirroring;
    }

    pub fn variant(&self) -> NamcoVariant {
        self.variant
    }

    pub fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    pub fn write_protect(&self) -> u8 {
        self.write_protect
    }

    pub fn low_chr_nt_mode(&self) -> bool {
        self.low_chr_nt_mode
    }

    pub fn high_chr_nt_mode(&self) -> bool {
        self.high_chr_nt_mode
    }

    pub fn set_write_protect(&mut self, value: u8) {
        self.write_protect = value;
    }

    pub fn set_chr_nt_modes(&mut self, low_chr_nt_mode: bool, high_chr_nt_mode: bool) {
        self.low_chr_nt_mode = low_chr_nt_mode;
        self.high_chr_nt_mode = high_chr_nt_mode;
    }

    pub fn set_mirroring(&mut self, mirroring: Mirroring) {
        self.mirroring = mirroring;
    }

    pub fn set_variant(&mut self, variant: NamcoVariant) -> bool {
        if !self.auto_detect_variant {
            return false;
        }
        if self.not_namco340 && variant == NamcoVariant::Namco340 {
            return false;
        }

        let changed = self.variant != variant;
        self.variant = variant;
        changed
    }

    pub fn on_prg_ram_access(&mut self) -> bool {
        self.not_namco340 = true;
        if self.variant == NamcoVariant::Namco340 {
            return self.set_variant(NamcoVariant::Unknown);
        }
        false
    }
}
