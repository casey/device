use super::*;

#[derive(Clone, Copy, ValueEnum)]
#[allow(clippy::arbitrary_source_item_ordering)]
pub(crate) enum Program {
  Hello,
  Busy,
  Noise,
  Expo,
  Transit,
}

impl Program {
  pub(crate) fn add_source(self, config: &Config, tap: &mut Tap) -> Result {
    match self {
      Self::Hello => {
        let wave = tap.load_wave(&config.find_song("old generic boss")?)?;
        tap.sequence_wave(&wave, 0.0, 0.0);
      }
      Self::Busy => Score::BusySignal.sequence(tap),
      Self::Noise => Score::BrownNoise.sequence(tap),
      Self::Expo => {
        let wave = tap.load_wave(&config.find_song("expo 2000 vocode")?)?;
        tap.sequence_wave(&wave, 0.0, 1.0);
      }
      Self::Transit => {
        let wave = tap.load_wave(&config.find_song("in transit corrente")?)?;
        tap.sequence_wave(&wave, 0.0, 0.0);
      }
    }
    Ok(())
  }

  pub(crate) fn scene(self) -> Scene {
    match self {
      Self::Hello => Scene::Hello,
      Self::Busy => Scene::Highwaystar,
      Self::Noise => Scene::Noise,
      Self::Expo => Scene::Starburst,
      Self::Transit => Scene::Kaleidoscope,
    }
  }
}
