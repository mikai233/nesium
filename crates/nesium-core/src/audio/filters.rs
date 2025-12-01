use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StereoFilterType {
    None,
    Delay,
    Panning,
    Comb,
}

#[derive(Debug, Default, Clone)]
pub struct StereoDelayState {
    pub(crate) last_delay_samples: usize,
    pub(crate) delayed_left: VecDeque<f32>,
    pub(crate) delayed_right: VecDeque<f32>,
}

impl StereoDelayState {
    pub(crate) fn apply(&mut self, samples: &mut [f32], sample_rate: f32, delay_ms: f32) {
        if delay_ms <= 0.0 || sample_rate <= 0.0 {
            return;
        }
        let frames = samples.len() / 2;
        if frames == 0 {
            return;
        }

        let delay_samples = ((delay_ms / 1000.0) * sample_rate) as usize;
        if delay_samples == 0 {
            return;
        }

        if delay_samples != self.last_delay_samples {
            self.delayed_left.clear();
            self.delayed_right.clear();
        }
        self.last_delay_samples = delay_samples;

        for i in 0..frames {
            let l = samples[2 * i];
            let r = samples[2 * i + 1];
            self.delayed_left.push_back(l);
            self.delayed_right.push_back(r);
        }

        if self.delayed_left.len() > delay_samples {
            let extra = self.delayed_left.len().saturating_sub(delay_samples);
            let samples_to_insert = extra.max(frames);
            let start = frames.saturating_sub(samples_to_insert);

            for i in start..frames {
                let idx = i * 2;
                let mono = 0.5 * (samples[idx] + samples[idx + 1]);
                let dl = self.delayed_left.pop_front().unwrap_or(0.0);
                let dr = self.delayed_right.pop_front().unwrap_or(0.0);
                let delayed = 0.5 * (dl + dr);

                samples[idx] = mono;
                samples[idx + 1] = delayed;
            }
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct StereoPanningState {
    left_factor: f32,
    right_factor: f32,
    last_angle_rad: f32,
}

impl StereoPanningState {
    fn update_factors(&mut self, angle_rad: f32) {
        const BASE: f32 = 0.707_106_77; // sqrt(2)/2
        let c = angle_rad.cos();
        let s = angle_rad.sin();
        self.left_factor = BASE * (c - s);
        self.right_factor = BASE * (c + s);
    }

    pub(crate) fn apply(&mut self, samples: &mut [f32], angle_deg: f32) {
        if angle_deg == 0.0 {
            return;
        }
        let frames = samples.len() / 2;
        if frames == 0 {
            return;
        }

        let angle_rad = angle_deg.to_radians();
        if (angle_rad - self.last_angle_rad).abs() > f32::EPSILON {
            self.update_factors(angle_rad);
            self.last_angle_rad = angle_rad;
        }

        for i in 0..frames {
            let idx = i * 2;
            let l = samples[idx];
            let r = samples[idx + 1];

            let out_l = (self.left_factor * l + self.left_factor * r) * 0.5;
            let out_r = (self.right_factor * r + self.right_factor * l) * 0.5;

            samples[idx] = out_l;
            samples[idx + 1] = out_r;
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct StereoCombState {
    pub(crate) last_delay_samples: usize,
    pub(crate) delayed_left: VecDeque<f32>,
    pub(crate) delayed_right: VecDeque<f32>,
}

impl StereoCombState {
    pub(crate) fn apply(
        &mut self,
        samples: &mut [f32],
        sample_rate: f32,
        delay_ms: f32,
        strength: f32,
    ) {
        if delay_ms <= 0.0 || sample_rate <= 0.0 || strength <= 0.0 {
            return;
        }

        let frames = samples.len() / 2;
        if frames == 0 {
            return;
        }

        let delay_samples = ((delay_ms / 1000.0) * sample_rate) as usize;
        if delay_samples == 0 {
            return;
        }

        if delay_samples != self.last_delay_samples {
            self.delayed_left.clear();
            self.delayed_right.clear();
            for _ in 0..delay_samples {
                self.delayed_left.push_back(0.0);
                self.delayed_right.push_back(0.0);
            }
        }
        self.last_delay_samples = delay_samples;

        let ratio = strength.clamp(0.0, 1.0);
        for i in 0..frames {
            let idx = i * 2;
            let l = samples[idx];
            let r = samples[idx + 1];

            self.delayed_left.push_back(l);
            self.delayed_right.push_back(r);

            let dl = self.delayed_left.front().copied().unwrap_or(0.0);
            let dr = self.delayed_right.front().copied().unwrap_or(0.0);
            let delayed = 0.5 * (dl + dr);
            let mono = 0.5 * (l + r);

            samples[idx] = mono + delayed * ratio;
            samples[idx + 1] = mono - delayed * ratio;

            self.delayed_left.pop_front();
            self.delayed_right.pop_front();
        }
    }
}

