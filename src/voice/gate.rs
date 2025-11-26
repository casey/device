use super::*;

pub(crate) struct Gate<T> {
  pub(crate) after: u64,
  pub(crate) inner: T,
}

impl<T> Gate<T> {
  fn new(inner: T, after: f32) -> Self {
    Self {
      inner,
      after: (after * 48_000.0) as u64,
    }
  }
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
