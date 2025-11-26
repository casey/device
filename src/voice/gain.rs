use super::*;

pub(crate) struct Gain<T> {
  pub(crate) gain: f32,
  pub(crate) inner: T,
}

impl<T: Voice> Voice for Gain<T> {
  fn sample(&mut self) -> Option<f32> {
    Some(self.inner.sample()? * self.gain)
  }
}
