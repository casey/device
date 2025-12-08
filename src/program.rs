use super::*;

#[derive(Clone, Copy, ValueEnum)]
#[allow(clippy::arbitrary_source_item_ordering)]
pub(crate) enum Program {
  Hello,
  Busy,
  Noise,
  Expo,
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
    }
    Ok(())
  }

  pub(crate) fn state(self) -> State {
    match self {
      Self::Hello => Scene::Hello.state(),
      Self::Busy => Scene::Highwaystar.state(),
      Self::Noise => Scene::Noise.state(),
      Self::Expo => Scene::Starburst.state(),
    }
  }
}
