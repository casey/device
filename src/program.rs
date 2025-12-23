use super::*;

#[derive(Clone, Copy, ValueEnum)]
#[allow(clippy::arbitrary_source_item_ordering)]
pub(crate) enum Program {
  Hello,
  Busy,
  Noise,
  Expo,
  Transit,
  Radio,
  Blaster,
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
      Self::Radio => {
        let wave = tap.load_wave(&config.find_song("next sun")?)?;
        tap.sequence_wave(&wave, 0.0, 0.0);
      }
      Self::Blaster => {
        let track = tap.load_track(&config.find_song("total 4/13 maria")?)?;
        tap.sequence_track(&track, 0.0, 0.0);
      }
    }
    Ok(())
  }

  pub(crate) fn db(self) -> Option<f32> {
    match self {
      Self::Radio => Some(-10.0),
      Self::Blaster => Some(-15.0),
      _ => None,
    }
  }

  pub(crate) fn fit(self) -> Option<bool> {
    match self {
      Self::Blaster => Some(true),
      _ => None,
    }
  }

  pub(crate) fn scene(self) -> Scene {
    match self {
      Self::Hello => Scene::Hello,
      Self::Busy => Scene::Highwaystar,
      Self::Noise => Scene::Noise,
      Self::Expo => Scene::Starburst,
      Self::Transit => Scene::Kaleidoscope,
      Self::Radio => Scene::BlackHole,
      Self::Blaster => Scene::None,
    }
  }

  pub(crate) fn script(self) -> Option<Script> {
    match self {
      Self::Blaster => Some(maria::script()),
      _ => None,
    }
  }
}
