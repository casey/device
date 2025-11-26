use super::*;

pub(crate) struct Sine {
  pub(crate) timer: Timer,
  pub(crate) frequency: f32,
}

impl Sine {
  pub(crate) fn new(frequency: f32) -> Self {
    Self {
      frequency,
      timer: Timer::default(),
    }
  }
}

impl Voice for Sine {
  fn reset(&mut self) {
    self.timer.reset();
  }

  fn sample(&mut self) -> Option<f32> {
    Some((self.timer.tick() * self.frequency * f32::consts::TAU).sin())
  }
}
