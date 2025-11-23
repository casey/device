use super::*;

pub(crate) struct WhiteNoise {
  distribution: Uniform<f32>,
  rng: SmallRng,
}

impl WhiteNoise {
  pub(crate) fn new() -> Self {
    Self {
      distribution: distribution(),
      rng: SmallRng::seed_from_u64(0),
    }
  }
}

impl Voice for WhiteNoise {
  fn sample(&mut self, _t: f32) -> f32 {
    self.rng.sample(self.distribution)
  }
}
