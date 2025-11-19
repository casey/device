use super::*;

#[derive(Clone, Default)]
pub(crate) struct Sound {
  pub(crate) samples: Vec<f32>,
  pub(crate) channels: u16,
  pub(crate) sample_rate: u32,
}

impl Sound {
  pub(crate) fn new(samples: Vec<f32>, channels: u16, sample_rate: u32) -> Self {
    Self {
      samples,
      channels,
      sample_rate,
    }
  }

  pub(crate) fn samples(&self) -> &[f32] {
    &self.samples
  }
}
