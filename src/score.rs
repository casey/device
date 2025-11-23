use super::*;

#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum Score {
  BusySignal,
  Silence,
  WhiteNoise,
}

impl Score {
  pub(crate) fn synthesizer(self) -> Synthesizer {
    match self {
      Self::BusySignal => Synthesizer::new(vec![
        Voice::Sine {
          frequency: 480.0,
          duty: 0.5,
        },
        Voice::Sine {
          frequency: 620.0,
          duty: 0.5,
        },
      ]),
      Self::Silence => Synthesizer::new(Vec::new()),
      Self::WhiteNoise => Synthesizer::new(vec![Voice::WhiteNoise {
        distribution: Uniform::new_inclusive((-1.0f32).next_up(), 1.0f32.next_down()).unwrap(),
        rng: SmallRng::seed_from_u64(0),
      }]),
    }
  }
}
