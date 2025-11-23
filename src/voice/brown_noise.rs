use super::*;

pub(crate) struct BrownNoise {
  distribution: Uniform<f32>,
  gain: f32,
  rng: SmallRng,
  state: f32,
}

impl BrownNoise {
  pub(crate) fn new() -> Self {
    Self {
      rng: SmallRng::seed_from_u64(0),
      state: 0.0,
      distribution: distribution(),
      gain: 0.02,
    }
  }
}

impl Voice for BrownNoise {
  fn sample(&mut self, _: f32) -> f32 {
    let sample = self.rng.sample(self.distribution);
    self.state += sample * self.gain;
    self.state = self.state.clamp(-1.0, 1.0);
    self.state
  }
}
