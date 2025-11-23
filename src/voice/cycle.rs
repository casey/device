use super::*;

pub(crate) struct Cycle<T> {
  pub(crate) inner: T,
  pub(crate) period: f32,
}

impl<T: Voice> Voice for Cycle<T> {
  fn sample(&mut self, t: f32) -> f32 {
    self.inner.sample(t % self.period)
  }
}
