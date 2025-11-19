use super::*;

#[derive(Clone, Default)]
pub(crate) struct Sound {
  pub(crate) samples: Vec<f32>,
  pub(crate) channels: u16,
  pub(crate) sample_rate: u32,
}

impl Sound {
  pub(crate) fn duration_micros(&self) -> u128 {
    if self.channels == 0 || self.sample_rate == 0 {
      return 0;
    }
    self.samples.len().into_u128() / u128::from(self.channels) * 1_000_000
      / u128::from(self.sample_rate)
  }
}
