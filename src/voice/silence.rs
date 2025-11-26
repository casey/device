use super::*;

pub(crate) struct Silence;

impl Voice for Silence {
  fn reset(&mut self) {}

  fn sample(&mut self) -> Option<f32> {
    Some(0.0)
  }
}
