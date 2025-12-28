use super::sprite::SpriteAttributes;
use crate::mem_block::MemBlock;

/// A single sprite slot for the current scanline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct SpriteSlot {
    /// Pattern bitplane 0 (shifted left once per dot after X expires).
    pattern_low: u8,
    /// Pattern bitplane 1 (shifted left once per dot after X expires).
    pattern_high: u8,
    /// Latched attributes (palette select, priority, flips).
    attributes: SpriteAttributes,
    /// X counter delaying sprite visibility.
    x_counter: u8,
    /// Indicates this slot belongs to OAM sprite 0.
    sprite0: bool,
}

impl Default for SpriteSlot {
    fn default() -> Self {
        Self {
            pattern_low: 0,
            pattern_high: 0,
            attributes: SpriteAttributes::empty(),
            x_counter: 0,
            sprite0: false,
        }
    }
}

/// Sprite pixel information produced by the pipeline for a single dot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub(crate) struct SpritePixel {
    /// Sprite palette select (0..=3).
    pub(crate) palette: u8,
    /// Sprite color index within the palette (0..=3, 0 means transparent).
    pub(crate) color: u8,
    /// Whether the sprite has background priority (is drawn behind).
    pub(crate) priority_behind_bg: bool,
    /// Whether this pixel came from sprite 0.
    pub(crate) is_sprite0: bool,
}

/// Sprite pixel pipeline for the current scanline.
///
/// The NES PPU has space for eight sprites per scanline. Each sprite has two
/// pattern shifters and an X counter. When the counter reaches zero, the
/// shifters begin outputting and advancing once per dot.
type SpriteSlots = MemBlock<SpriteSlot, 8>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct SpritePipeline {
    slots: SpriteSlots,
    active_count: u8,
}

impl Default for SpritePipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl SpritePipeline {
    /// Creates a new pipeline with no active sprites.
    pub(crate) fn new() -> Self {
        Self {
            slots: SpriteSlots::new(),
            active_count: 0,
        }
    }

    /// Clears all active sprite shifters.
    pub(crate) fn clear(&mut self) {
        self.slots.fill(SpriteSlot::default());
        self.active_count = 0;
    }

    /// Loads sprite data for the next scanline from the evaluation/fetch stage.
    ///
    /// `count` is the number of sprites that were found for the scanline (0..=8).
    /// `sprite0_in_range` indicates whether sprite 0 was one of them.
    ///
    /// Pattern bytes are optionally bit-reversed when horizontal flip is set so
    /// that shifting left always walks pixels left-to-right on output.
    pub(crate) fn load_scanline(
        &mut self,
        count: u8,
        sprite0_in_range: bool,
        attrs: &[u8],
        xs: &[u8],
        pattern_low: &[u8],
        pattern_high: &[u8],
    ) {
        self.clear();
        self.active_count = count.min(8);

        for i in 0..self.active_count as usize {
            let mut low = pattern_low[i];
            let mut high = pattern_high[i];
            let attributes = SpriteAttributes::from_bits_retain(attrs[i]);

            // Pre-flip bitplanes when horizontal flip is set.
            if attributes.contains(SpriteAttributes::FLIP_HORIZONTAL) {
                low = low.reverse_bits();
                high = high.reverse_bits();
            }

            self.slots[i] = SpriteSlot {
                pattern_low: low,
                pattern_high: high,
                attributes,
                x_counter: xs[i],
                sprite0: sprite0_in_range && i == 0,
            };
        }
    }

    /// Samples the current sprite pixel and advances active shifters by one dot.
    pub(crate) fn sample_and_shift(&mut self) -> SpritePixel {
        let mut chosen: Option<SpritePixel> = None;

        for slot in self.slots.iter_mut().take(self.active_count as usize) {
            // Hardware order: decrement X counter first; when it transitions to
            // zero, the shifter starts outputting on the *next* dot.
            if slot.x_counter > 0 {
                slot.x_counter = slot.x_counter.saturating_sub(1);
                continue;
            }

            // Extract the current pixel from the MSB of each bitplane.
            let bit0 = (slot.pattern_low >> 7) & 1;
            let bit1 = (slot.pattern_high >> 7) & 1;
            let color = (bit1 << 1) | bit0;

            if chosen.is_none() && color != 0 {
                let palette = slot.attributes.bits() & 0b11;
                let priority_behind_bg = slot
                    .attributes
                    .contains(SpriteAttributes::PRIORITY_BEHIND_BACKGROUND);
                chosen = Some(SpritePixel {
                    palette,
                    color,
                    priority_behind_bg,
                    is_sprite0: slot.sprite0,
                });
            }

            // Advance shifters once per dot after the delay has expired.
            slot.pattern_low <<= 1;
            slot.pattern_high <<= 1;
        }

        chosen.unwrap_or_default()
    }

    pub(crate) fn save_state(&self) -> crate::ppu::savestate::SpritePipelineState {
        let mut slots = [crate::ppu::savestate::SpriteSlotState::default(); 8];
        for (idx, slot) in self.slots.iter().enumerate() {
            slots[idx] = crate::ppu::savestate::SpriteSlotState {
                pattern_low: slot.pattern_low,
                pattern_high: slot.pattern_high,
                attributes: slot.attributes.bits(),
                x_counter: slot.x_counter,
                sprite0: slot.sprite0,
            };
        }
        crate::ppu::savestate::SpritePipelineState {
            active_count: self.active_count,
            slots,
        }
    }

    pub(crate) fn load_state(&mut self, state: crate::ppu::savestate::SpritePipelineState) {
        self.active_count = state.active_count.min(8);
        for (idx, slot_state) in state.slots.iter().enumerate() {
            self.slots[idx] = SpriteSlot {
                pattern_low: slot_state.pattern_low,
                pattern_high: slot_state.pattern_high,
                attributes: SpriteAttributes::from_bits_retain(slot_state.attributes),
                x_counter: slot_state.x_counter,
                sprite0: slot_state.sprite0,
            };
        }
    }
}
