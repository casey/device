use super::*;

#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum Score {
  BrownNoise,
  BusySignal,
  ClickTrack,
  PinkNoise,
  Silence,
  WhiteNoise,
}

impl Score {
  pub(crate) fn synthesizer(self) -> Synthesizer {
    match self {
      Self::BrownNoise => voice::BrownNoise::new().gain(0.125).into(),
      Self::BusySignal => voice::Cycle {
        inner: voice::Gate {
          after: 0.5,
          inner: voice::Sum::new()
            .add(voice::Sine { frequency: 480.0 })
            .add(voice::Sine { frequency: 620.0 }),
        },
        period: 1.0,
      }
      .gain(0.125)
      .into(),
      Self::ClickTrack => voice::Cycle {
        period: 2.0 / 3.0,
        inner: voice::Envelope {
          attack: 0.001,
          decay: 0.02,
          sustain: 0.000,
          release: 0.002,
          inner: voice::BrownNoise::new(),
        },
      }
      .into(),
      Self::PinkNoise => voice::PinkNoise::new().gain(0.125).into(),
      Self::Silence => voice::Silence.into(),
      Self::WhiteNoise => voice::WhiteNoise::new().gain(0.125).into(),
    }
  }
}
