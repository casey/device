use super::*;

pub(crate) struct Envelope<T> {
  pub(crate) attack: f32,
  pub(crate) decay: f32,
  pub(crate) inner: T,
  pub(crate) release: f32,
  pub(crate) sustain: f32,
}

impl<T: Voice> Voice for Envelope<T> {
  fn sample(&mut self, t: f32) -> f32 {
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
      return 0.0;
    };

    self.inner.sample(t) * scale
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  struct Constant {
    value: f32,
  }

  impl Voice for Constant {
    fn sample(&mut self, _t: f32) -> f32 {
      self.value
    }
  }

  #[test]
  fn envelope() {
    let mut envelope = Envelope {
      attack: 1.0,
      decay: 1.0,
      inner: Constant { value: 1.0 },
      release: 1.0,
      sustain: 1.0,
    };

    assert_eq!(envelope.sample(0.0), 0.0);
    assert_eq!(envelope.sample(0.5), 0.5);
    assert_eq!(envelope.sample(1.0), 1.0);
    assert_eq!(envelope.sample(1.5), 0.75);
    assert_eq!(envelope.sample(2.0), 0.5);
    assert_eq!(envelope.sample(2.5), 0.5);
    assert_eq!(envelope.sample(3.0), 0.5);
    assert_eq!(envelope.sample(3.5), 0.25);
    assert_eq!(envelope.sample(4.0), 0.0);
    assert_eq!(envelope.sample(4.5), 0.0);
  }
}
