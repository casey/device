use super::*;

pub(crate) struct Duty<T> {
  pub(crate) inner: T,
  pub(crate) off: f32,
  pub(crate) on: f32,
  pub(crate) timer: Timer,
}

impl<T: Voice> Voice for Duty<T> {
  fn reset(&mut self) {
    self.inner.reset();
    self.timer.reset();
  }

  fn sample(&mut self) -> Option<f32> {
    let sample = self.inner.sample()?;
    let t = self.timer.tick() % (self.on + self.off);

    if t < self.on { Some(sample) } else { Some(0.0) }
  }
}
