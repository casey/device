use {super::*, all_night::AllNight};

mod all_night;
mod maria;

#[derive(Clone, Copy, ValueEnum)]
#[allow(clippy::arbitrary_source_item_ordering)]
pub(crate) enum Program {
  Hello,
  Busy,
  Noise,
  Expo,
  Transit,
  Radio,
  Maria,
  AllNight,
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
      Self::Maria => {
        let track = tap.load_track(&config.find_song("total 4/13 maria")?)?;
        tap.sequence_track(&track, 0.0, 0.0);
      }
      Self::AllNight => {
        let track = tap.load_track(&config.find_song("romare.*all night")?)?;
        tap.sequence_track(&track, 0.0, 0.0);
      }
    }
    Ok(())
  }

  pub(crate) fn script(self) -> Option<Script> {
    match self {
      Self::Maria => Some(maria::script()),
      _ => None,
    }
  }

  pub(crate) fn state(self, rng: &mut SmallRng) -> State {
    match self {
      Self::Hello => Scene::Hello.state(rng),
      Self::Busy => Scene::Highwaystar.state(rng),
      Self::Noise => Scene::Noise.state(rng),
      Self::Expo => Scene::Starburst.state(rng),
      Self::Transit => Scene::Kaleidoscope.state(rng),
      Self::Radio => {
        let mut state = Scene::BlackHole.state(rng);
        state.db = -10.0;
        state
      }
      Self::Maria => {
        let mut state = Scene::None.state(rng);
        state.db = -15.0;
        state.interpolate = true;
        state
      }
      Self::AllNight => State {
        callback: Some(Box::new(AllNight::default())),
        ..default()
      },
    }
  }
}
