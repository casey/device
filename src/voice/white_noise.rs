use super::*;

pub(crate) struct WhiteNoise {
  distribution: Uniform<f32>,
  rng: SmallRng,
  seed: [u8; 32],
}

impl WhiteNoise {
  pub(crate) fn new() -> Self {
    let seed = rand::rng().random();
    Self {
      distribution: distribution(),
      rng: SmallRng::from_seed(seed),
      seed,
    }
  }
}

impl Voice for WhiteNoise {
  fn reset(&mut self) {
    self.rng = SmallRng::from_seed(self.seed);
  }

  fn sample(&mut self) -> Option<f32> {
    self.rng.sample(self.distribution)
  }
}
