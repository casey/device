use {super::*, all_night::AllNight, position::bbq};

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
  Suplex,
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
      Self::Suplex => {
        let wave = tap.load_wave(&config.find_song("orange evening$")?)?;
        let track = Track {
          wave,
          tempo: Tempo {
            bpm: 118.0,
            offset: 0.133,
          },
        };
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

  pub(crate) fn state(self, config: &Config, rng: &mut SmallRng) -> Result<State> {
    let state = match self {
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
        callback: Some(Box::new(AllNight::new())),
        ..default()
      },
      Self::Suplex => {
        let path = &config.find_image(r"nichijou-principal-german-suplex-deer")?;

        let reader = BufReader::new(File::open(path).context(error::FilesystemIo { path })?);
        let decoder = GifDecoder::new(reader).context(error::ImageDecode { path })?;

        let mut media = Vec::new();

        for frame in decoder.into_frames() {
          let frame = frame.context(error::ImageDecode { path })?;
          let buffer = frame.into_buffer();

          let width = buffer.width();
          let height = buffer.height();

          let image = ImageData {
            alpha_type: peniko::ImageAlphaType::Alpha,
            data: buffer.into_vec().into(),
            format: peniko::ImageFormat::Rgba8,
            height,
            width,
          };

          media.push(Media::new().image(image).handle());
        }

        let mut state = State::new();

        state.filters.push(Filter {
          blend_mode: BlendMode::Source,
          media: Some(media[0].clone()),
          ..default()
        });

        let mut index = 0;

        state.callback(move |state, tick| {
          let Some(position) = tick.position else {
            return;
          };

          let tempo = tick.tempo.unwrap();

          if position < bbq(19, 1, 1) {
            if tick.advanced() && position.is_phrase() {
              state.filters[0].media = Some(media[index % media.len()].clone());
              index += 1;
            }
          } else if position < bbq(23, 1, 1) {
          } else if position < bbq(24, 1, 1) {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let frame =
              index + (tempo.bars(tick.time).fract() * (media.len() - index) as f64) as usize;
            state.filters[0].media = Some(media[frame.min(media.len() - 1)].clone());
          } else {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let frame = (tempo.bars(tick.time).fract() * media.len() as f64) as usize;
            state.filters[0].media = Some(media[frame.min(media.len() - 1)].clone());
          }
        });

        state
      }
    };

    Ok(state)
  }
}
