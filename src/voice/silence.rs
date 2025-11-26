use super::*;

pub(crate) struct Silence;

impl Voice for Silence {
  fn sample(&mut self) -> Option<f32> {
    Some(0.0)
  }
}
