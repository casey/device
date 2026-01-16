use super::*;

pub(crate) struct Analyzer {
  complex_frequencies: Vec<Complex<f32>>,
  frequencies: Vec<f32>,
  planner: FftPlanner<f32>,
  rms: f32,
  samples: Vec<f32>,
  scratch: Vec<Complex<f32>>,
}

impl Analyzer {
  pub(crate) fn frequencies(&self) -> &[f32] {
    &self.frequencies
  }

  pub(crate) fn new() -> Self {
    Self {
      complex_frequencies: Vec::new(),
      frequencies: Vec::new(),
      planner: FftPlanner::new(),
      rms: 0.0,
      samples: Vec::new(),
      scratch: Vec::new(),
    }
  }

  pub(crate) fn rms(&self) -> f32 {
    self.rms
  }

  pub(crate) fn samples(&self) -> &[f32] {
    &self.samples
  }

  pub(crate) fn update(&mut self, sound: &Sound, done: bool, state: &State) {
    if done {
      self.samples.clear();
    } else {
      let old = self.samples.len();
      self.samples.extend(sound.downmix());
      self
        .samples
        .drain(..self.samples.len().saturating_sub(128).min(old));
    }

    let samples = &self.samples[..self.samples.len() & !1];

    self.complex_frequencies.clear();
    self
      .complex_frequencies
      .extend(samples.iter().map(Complex::from));
    let fft = self.planner.plan_fft_forward(samples.len());
    let scratch_len = fft.get_inplace_scratch_len();
    if self.scratch.len() < scratch_len {
      self.scratch.resize(scratch_len, 0.0.into());
    }
    fft.process_with_scratch(
      &mut self.complex_frequencies,
      &mut self.scratch[..scratch_len],
    );

    let n = self.complex_frequencies.len();
    let half = n / 2;
    let spacing = sound.format().sample_rate as f32 / n as f32;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let threshold = (state.bandpass.x / spacing) as usize;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let cutoff = (state.bandpass.y / spacing) as usize;

    self.frequencies.clear();
    self.frequencies.extend(
      self
        .complex_frequencies
        .iter()
        .enumerate()
        .skip(threshold)
        .take(cutoff.min(half).saturating_sub(threshold))
        .map(|(i, c)| {
          let weight = if i == 0 || i == half { 1.0 } else { 2.0 };
          c.norm() * weight
        }),
    );

    let rms = (self.frequencies.iter().map(|&f| f * f).sum::<f32>()
      / self.frequencies.len().max(1) as f32)
      .sqrt();

    let alpha = if rms > self.rms {
      state.alpha
    } else {
      state.alpha / 5.0
    };

    self.rms = alpha * rms + (1.0 - alpha) * self.rms;
  }
}
