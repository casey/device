use super::*;

pub(crate) struct Constant {
  pub(crate) value: f32,
}

impl Voice for Constant {
  fn sample(&mut self, _t: f32) -> f32 {
    self.value
  }
}
