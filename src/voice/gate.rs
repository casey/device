use super::*;

pub(crate) struct Gate<T> {
  pub(crate) after: f32,
  pub(crate) inner: T,
}

impl<T: Voice> Voice for Gate<T> {
  fn sample(&mut self, t: f32) -> f32 {
    if t < self.after {
      self.inner.sample(t)
    } else {
      0.0
    }
  }
}
