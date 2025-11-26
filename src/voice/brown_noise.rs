use super::*;

pub(crate) struct BrownNoise {
  gain: f32,
  state: f32,
  inner: WhiteNoise,
}

impl BrownNoise {
  pub(crate) fn new() -> Self {
    Self {
      gain: 0.015,
      inner: WhiteNoise::new(),
      state: 0.0,
    }
  }
}

impl Voice for BrownNoise {
  fn reset(&mut self) {
    self.inner.reset();
  }

  fn sample(&mut self) -> Option<f32> {
    let sample = self.inner.sample()?;
    self.state += sample * self.gain;
    self.state = self.state.clamp(-1.0, 1.0);
    Some(self.state)
  }
}
