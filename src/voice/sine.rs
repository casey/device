use super::*;

pub(crate) struct Sine {
  pub(crate) frequency: f32,
}

impl Voice for Sine {
  fn sample(&mut self, t: f32) -> f32 {
    (t * self.frequency * f32::consts::TAU).sin()
  }
}
