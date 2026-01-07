use {
  super::*,
  fundsp::prelude32::{saw_hz, sine_hz},
};

#[derive(Clone, Copy, Default)]
pub(crate) enum Patch {
  Saw,
  #[default]
  Sine,
}

impl Patch {
  pub(crate) fn sequence(self, semitones: u8, tap: &mut Tap) {
    let frequency = 261.63 * 2.0f32.powf(semitones as f32 / 12.0);
    match self {
      Self::Saw => tap.sequence(saw_hz(frequency) * 0.25, 0.3, 0.05, 0.05),
      Self::Sine => tap.sequence(sine_hz(frequency) * 0.25, 0.3, 0.05, 0.05),
    }
  }
}
