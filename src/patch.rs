use super::*;

#[derive(Clone, Copy, Default)]
pub(crate) enum Patch {
  Saw,
  #[default]
  Sine,
}

impl Patch {
  pub(crate) fn add(self, semitones: u8, tap: &Tap) {
    let frequency = 261.63 * 2.0f32.powf(semitones as f32 / 12.0);

    match self {
      Self::Saw => {
        tap.add(
          voice::Saw::new(frequency)
            .envelope(0.001, 0.1, 0.2, 0.1)
            .gain(0.25)
            .emitter(),
        );
      }
      Self::Sine => {
        tap.add(
          voice::Sine::new(frequency)
            .envelope(0.001, 0.1, 0.2, 0.1)
            .gain(0.25)
            .emitter(),
        );
      }
    }
  }
}
