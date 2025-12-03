use super::*;

#[derive(Clone, Copy, ValueEnum)]
#[allow(clippy::arbitrary_source_item_ordering)]
pub(crate) enum Program {
  Hello,
  Busy,
  Noise,
}

impl Program {
  pub(crate) fn add_source(self, config: &Config, tap: &mut Tap) -> Result {
    match self {
      Self::Hello => {
        let wave = tap.load_wave(&config.find_song("old generic boss")?)?;
        tap.sequence_wave(&wave);
      }
      Self::Busy => Score::BusySignal.sequence(tap),
      Self::Noise => Score::BrownNoise.sequence(tap),
    }
    Ok(())
  }

  pub(crate) fn state(self, rng: &mut SmallRng) -> State {
    match self {
      Self::Hello => Scene::Hello.state(rng),
      Self::Busy => Scene::Highwaystar.state(rng),
      Self::Noise => Scene::Noise.state(rng),
    }
  }
}
