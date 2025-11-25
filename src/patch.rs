use super::*;

#[derive(Clone, Copy, Default)]
pub(crate) enum Patch {
  #[default]
  Sine,
  Saw,
}

impl Patch {
  pub(crate) fn add(self, semitones: u8, mixer: &Mixer) {
    let frequency = 261.63 * 2.0f32.powf(semitones as f32 / 12.0);

    match self {
      Self::Sine => {
        mixer.add(
          voice::Sine { frequency }
            .envelope(0.001, 0.1, 0.2, 0.1)
            .gain(0.25)
            .emitter(),
        );
      }
      Self::Saw => {
        mixer.add(
          voice::Saw { frequency }
            .envelope(0.001, 0.1, 0.2, 0.1)
            .gain(0.25)
            .emitter(),
        );
      }
    }
  }
}
