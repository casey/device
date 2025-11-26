use super::*;

pub(crate) struct Gate<T> {
  pub(crate) after: f32,
  pub(crate) inner: T,
  pub(crate) timer: Timer,
}

impl<T: Voice> Voice for Gate<T> {
  fn reset(&mut self) {
    self.inner.reset();
    self.timer.reset();
  }

  fn sample(&mut self) -> Option<f32> {
    let t = self.timer.tick();

    if t < self.after {
      self.inner.sample()
    } else {
      None
    }
  }
}
