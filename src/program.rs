use super::*;

#[derive(Clone, Copy, ValueEnum)]
#[allow(clippy::arbitrary_source_item_ordering)]
pub(crate) enum Program {
  Hello,
  Busy,
  Noise,
}

impl Program {
  pub(crate) fn add_source(self, config: &Config, tap: &Tap) -> Result {
    match self {
      Self::Hello => {
        let wave = tap.load_wave(&config.find_song("old generic boss")?)?;
        tap.sequence_wave(wave);
      }
      Self::Busy => Score::BusySignal.sequence(tap),
      Self::Noise => Score::BrownNoise.sequence(tap),
    }
    Ok(())
  }

  pub(crate) fn state(self) -> State {
    match self {
      Self::Hello => Scene::Hello.state(),
      Self::Busy => Scene::Highwaystar.state(),
      Self::Noise => Scene::Noise.state(),
    }
  }
}
