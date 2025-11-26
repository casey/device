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
      Self::Hello => tap.add(open_song(&config.find_song("old generic boss")?)?),
      Self::Busy => tap.add(Score::BusySignal.source()),
      Self::Noise => tap.add(Score::BrownNoise.source()),
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
