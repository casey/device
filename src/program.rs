use super::*;

#[derive(Clone, Copy, ValueEnum)]
#[allow(clippy::arbitrary_source_item_ordering)]
pub(crate) enum Program {
  Hello,
  Busy,
  Noise,
}

impl Program {
  pub(crate) fn state(self) -> State {
    match self {
      Self::Hello => Scene::Hello.state(),
      Self::Busy => Scene::Highwaystar.state(),
      Self::Noise => Scene::Noise.state(),
    }
  }

  pub(crate) fn stream(self, config: &Config) -> Result<Box<dyn Stream>> {
    match self {
      Self::Hello => Ok(Box::new(Track::new(
        &config.find_song("old generic boss")?,
      )?)),
      Self::Busy => Ok(Box::new(Score::BusySignal.synthesizer())),
      Self::Noise => Ok(Box::new(Score::BrownNoise.synthesizer())),
    }
  }
}
