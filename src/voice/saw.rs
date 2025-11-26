use super::*;

pub(crate) struct Saw {
  pub(crate) frequency: f32,
  pub(crate) timer: Timer,
}

impl Saw {
  pub(crate) fn new(frequency: f32) -> Self {
    Saw {
      frequency,
      timer: Timer::default(),
    }
  }
}

impl Voice for Saw {
  fn reset(&mut self) {
    self.timer.reset();
  }

  fn sample(&mut self) -> Option<f32> {
    Some(self.frequency * self.timer.tick() % 1.0 * 2.0 - 1.0)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn saw() {
    let mut saw = Saw::new(1.0);
    assert_eq!(saw.sample().unwrap(), -1.0);
    assert_eq!(saw.sample().unwrap(), -0.5);
    assert_eq!(saw.sample().unwrap(), 0.0);
    assert_eq!(saw.sample().unwrap(), 0.5);
    assert_eq!(saw.sample().unwrap(), -1.0);

    let mut saw = Saw::new(2.0);
    assert_eq!(saw.sample().unwrap(), -1.0);
    assert_eq!(saw.sample().unwrap(), -0.5);
    assert_eq!(saw.sample().unwrap(), 0.0);
    assert_eq!(saw.sample().unwrap(), 0.5);
    assert_eq!(saw.sample().unwrap(), -1.0);
  }
}
