use super::*;

pub(crate) struct Sine {
  timer: Timer,
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
  fn sample(&mut self) -> Option<f32> {
    Some((self.timer.next() * self.frequency * f32::consts::TAU).sin())
  }
}
