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
      Self::BusySignal => Synthesizer::new(
        voice::Cycle {
          inner: voice::Gate {
            after: 0.5,
            inner: voice::Sum::new()
              .add(voice::Sine { frequency: 480.0 })
              .add(voice::Sine { frequency: 620.0 }),
          },
          period: 1.0,
        }
        .gain(0.125),
      ),
      Self::ClickTrack => Synthesizer::new(voice::Cycle {
        period: 2.0 / 3.0,
        inner: voice::Envelope {
          attack: 0.001,  // 0-1ms
          decay: 0.010,   // 2-10ms
          sustain: 0.000, // 0ms
          release: 0.005, // 0-5ms
          inner: voice::BrownNoise::new(),
        },
      }),
      Self::Silence => Synthesizer::new(voice::Silence),
      Self::WhiteNoise => Synthesizer::new(voice::WhiteNoise::new().gain(0.125)),
      Self::PinkNoise => Synthesizer::new(voice::PinkNoise::new().gain(0.125)),
      Self::BrownNoise => Synthesizer::new(voice::BrownNoise::new().gain(0.125)),
    }
  }
}
