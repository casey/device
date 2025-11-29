use {
  super::*,
  fundsp::hacker32::{brown, constant, lfo, pink, ramp_hz, shape_fn, sine_hz, white},
};

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
  pub(crate) fn sequence(self, tap: &mut Tap) {
    match self {
      Self::BusySignal => tap.sequence_indefinite(
        (sine_hz(480.0) + sine_hz(620.0)) * lfo(|t| if t % 1.0 < 0.5 { 1.0 } else { 0.0 }) * 0.25,
      ),
      Self::BrownNoise => {
        tap.sequence_indefinite(brown() * 0.25);
      }

      Self::ClickTrack => tap.sequence_indefinite(
        (ramp_hz(2.0 / 3.0) >> shape_fn(|x: f32| if x < 0.010 { 1.0 } else { 0.0 }))
          * brown()
          * 0.5,
      ),
      Self::PinkNoise => {
        tap.sequence_indefinite(pink() * 0.25);
      }
      Self::Silence => tap.sequence_indefinite(constant(0.0)),
      Self::WhiteNoise => {
        tap.sequence_indefinite(white() * 0.25);
      }
    }
  }
}
