use super::*;

pub(crate) struct Saw {
  pub(crate) frequency: f32,
}

impl Voice for Saw {
  fn sample(&mut self, t: f32) -> f32 {
    self.frequency * t % 1.0 * 2.0 - 1.0
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn saw() {
    let mut saw = Saw { frequency: 1.0 };
    assert_eq!(saw.sample(0.00), -1.0);
    assert_eq!(saw.sample(0.25), -0.5);
    assert_eq!(saw.sample(0.50), 0.0);
    assert_eq!(saw.sample(0.75), 0.5);
    assert_eq!(saw.sample(1.00), -1.0);

    let mut saw = Saw { frequency: 2.0 };
    assert_eq!(saw.sample(0.000), -1.0);
    assert_eq!(saw.sample(0.125), -0.5);
    assert_eq!(saw.sample(0.250), 0.0);
    assert_eq!(saw.sample(0.375), 0.5);
    assert_eq!(saw.sample(0.500), -1.0);
  }
}
