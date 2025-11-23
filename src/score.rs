use super::*;

#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum Score {
  BusySignal,
  Silence,
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
    }
  }
}
