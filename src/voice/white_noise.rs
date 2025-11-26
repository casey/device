use super::*;

pub(crate) struct WhiteNoise {
  distribution: Uniform<f32>,
  rng: SmallRng,
}

impl WhiteNoise {
  pub(crate) fn new() -> Self {
    Self {
      distribution: distribution(),
      rng: SmallRng::from_rng(&mut rand::rng()),
    }
  }
}

impl Voice for WhiteNoise {
  fn sample(&mut self) -> Option<f32> {
    Some(self.rng.sample(self.distribution))
  }
}
