use super::*;

pub(crate) struct Gain<T> {
  pub(crate) gain: f32,
  pub(crate) inner: T,
}

impl<T: Voice> Voice for Gain<T> {
  fn sample(&mut self, t: f32) -> f32 {
    self.inner.sample(t) * self.gain
  }
}
