use super::*;

pub(crate) enum Voice {
  Sine { duty: f32, frequency: f32 },
}

impl Voice {
  pub(crate) fn sample(&self, t: f32) -> f32 {
    match self {
      Self::Sine { duty, frequency } => {
        if t.fract() < *duty {
          (t * *frequency * f32::consts::TAU).sin()
        } else {
          0.0
        }
      }
    }
  }
}
