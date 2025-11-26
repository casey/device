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
  pub(crate) fn source(self) -> Box<dyn Source + Send> {
    match self {
      Self::BrownNoise => voice::BrownNoise::new().gain(0.125).source(),
      Self::BusySignal => voice::Sum::new()
        .add(voice::Sine::new(480.0))
        .add(voice::Sine::new(620.0))
        .duty(0.5, 0.5)
        .gain(0.25)
        .source(),
      Self::ClickTrack => voice::BrownNoise::new()
        .envelope(0.001, 0.02, 0.000, 0.002)
        .cycle(2.0 / 3.0)
        .source(),
      Self::PinkNoise => voice::PinkNoise::new().gain(0.125).source(),
      Self::Silence => voice::Silence.source(),
      Self::WhiteNoise => voice::WhiteNoise::new().gain(0.125).source(),
    }
  }
}
