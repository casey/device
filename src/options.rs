use {
  super::*,
  clap::builder::styling::{AnsiColor, Effects, Styles},
};

const AUDIO: &str = "audio";

#[derive(Clone, Default, Parser)]
#[command(
  version,
  styles = Styles::styled()
    .error(AnsiColor::Red.on_default() | Effects::BOLD)
    .header(AnsiColor::Yellow.on_default() | Effects::BOLD)
    .invalid(AnsiColor::Red.on_default())
    .literal(AnsiColor::Blue.on_default())
    .placeholder(AnsiColor::Cyan.on_default())
    .usage(AnsiColor::Yellow.on_default() | Effects::BOLD)
    .valid(AnsiColor::Green.on_default())
)]
pub(crate) struct Options {
  #[arg(allow_hyphen_values = true, long)]
  pub(crate) db: Option<f32>,
  #[arg(long)]
  pub(crate) fps: Option<Fps>,
  #[arg(group = AUDIO, long)]
  pub(crate) input: bool,
  #[arg(long)]
  pub(crate) interpolate: bool,
  #[arg(long)]
  pub(crate) program: Option<Program>,
  #[arg(long)]
  pub(crate) resolution: Option<NonZeroU32>,
  #[arg(group = AUDIO, long)]
  pub(crate) score: Option<Score>,
  #[arg(group = AUDIO, long)]
  pub(crate) song: Option<String>,
  #[arg(group = AUDIO, long)]
  pub(crate) track: Option<Utf8PathBuf>,
  #[arg(long)]
  pub(crate) verbose: bool,
  #[arg(long)]
  pub(crate) volume: Option<f32>,
}

impl Options {
  fn find_song(song: &str) -> Result<Utf8PathBuf> {
    let song = RegexBuilder::new(song)
      .case_insensitive(true)
      .build()
      .context(error::SongRegex)?;

    let mut matches = Vec::<Utf8PathBuf>::new();

    let home = dirs::home_dir().context(error::Home)?;

    let music = home.join("Music/Music/Media.localized/Music");

    for entry in WalkDir::new(&music) {
      let entry = entry.context(error::SongWalk)?;

      if entry.file_type().is_dir() {
        continue;
      }

      let path = entry.path();

      let haystack = path.strip_prefix(&music).unwrap().with_extension("");

      let Some(haystack) = haystack.to_str() else {
        continue;
      };

      if song.is_match(haystack) {
        matches.push(path.into_utf8_path()?.into());
      }
    }

    if matches.len() > 1 {
      return Err(error::SongAmbiguous { matches }.build());
    }

    match matches.into_iter().next() {
      Some(path) => Ok(path),
      None => Err(error::SongMatch { song }.build()),
    }
  }

  pub(crate) fn state(&self) -> State {
    let mut state = self.program.map(Program::state).unwrap_or_default();
    if let Some(db) = self.db {
      state.db = db;
    }
    state.fps = self.fps.or(state.fps);
    state.interpolate = self.interpolate;
    state.resolution = self.resolution.or(state.resolution);
    state
  }

  pub(crate) fn stdio(&self) -> Stdio {
    if self.verbose {
      Stdio::inherit()
    } else {
      Stdio::piped()
    }
  }

  pub(crate) fn stream(&self) -> Result<Box<dyn Stream>> {
    if let Some(song) = &self.song {
      Ok(Box::new(Track::new(&Self::find_song(song)?)?))
    } else if let Some(score) = self.score {
      Ok(Box::new(score.synthesizer()))
    } else if let Some(track) = &self.track {
      Ok(Box::new(Track::new(track)?))
    } else {
      Ok(Box::new(Score::Silence.synthesizer()))
    }
  }
}
