use super::*;

#[allow(clippy::arbitrary_source_item_ordering)]
pub(crate) struct Envelope<T> {
  pub(crate) inner: T,
  pub(crate) timer: Timer,
  pub(crate) attack: f32,
  pub(crate) decay: f32,
  pub(crate) sustain: f32,
  pub(crate) release: f32,
}

impl<T: Voice> Voice for Envelope<T> {
  fn reset(&mut self) {
    self.inner.reset();
    self.timer.reset();
  }

  fn sample(&mut self) -> Option<f32> {
    let t = self.timer.tick();

    let a = self.attack;
    let d = self.decay;
    let s = self.sustain;
    let r = self.release;

    let scale = if t < a {
      t / a
    } else if t < a + d {
      f32::midpoint(-((t - a) / d - 1.0), 1.0)
    } else if t < a + d + s {
      0.5
    } else if t < a + d + s + r {
      -((t - a - d - s) / r - 1.0) / 2.0
    } else {
      return Some(0.0);
    };

    Some(self.inner.sample()? * scale)
  }
}
