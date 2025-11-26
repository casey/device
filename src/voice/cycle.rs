use super::*;

pub(crate) struct Cycle<T> {
  pub(crate) inner: T,
  pub(crate) period: u64,
  pub(crate) sample: u64,
}

impl<T: Voice> Voice for Cycle<T> {
  fn reset(&mut self) {
    self.inner.reset();
    self.sample = 0;
  }

  fn sample(&mut self) -> Option<f32> {
    if self.sample == self.period {
      self.reset();
    }

    self.inner.sample()
  }
}
