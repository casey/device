use super::*;

pub(crate) enum Voice {
  Sine {
    duty: f32,
    frequency: f32,
  },
  WhiteNoise {
    distribution: Uniform<f32>,
    rng: SmallRng,
  },
}

impl Voice {
  pub(crate) fn sample(&mut self, t: f32) -> f32 {
    match self {
      Self::Sine { duty, frequency } => {
        if t.fract() < *duty {
          (t * *frequency * f32::consts::TAU).sin()
        } else {
          0.0
        }
      }
      Self::WhiteNoise { distribution, rng } => rng.sample(*distribution),
    }
  }
}
