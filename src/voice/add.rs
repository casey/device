use super::*;

pub(crate) struct Add<A, B> {
  pub(crate) a: A,
  pub(crate) b: B,
}

impl<A: Voice, B: Voice> Voice for Add<A, B> {
  fn reset(&mut self) {
    self.a.reset();
    self.b.reset();
  }

  fn sample(&mut self) -> Option<f32> {
    match (self.a.sample(), self.b.sample()) {
      (None, None) => None,
      (Some(a), None) => Some(a),
      (None, Some(b)) => Some(b),
      (Some(a), Some(b)) => Some(a + b),
    }
  }
}
