use super::*;

pub(crate) struct Silence;

impl Voice for Silence {
  fn sample(&mut self, _t: f32) -> f32 {
    0.0
  }
}
