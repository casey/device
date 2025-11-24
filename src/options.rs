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
  pub(crate) interpolate: Option<bool>,
  #[arg(long)]
  pub(crate) program: Option<Program>,
  #[arg(long)]
  pub(crate) resolution: Option<NonZeroU32>,
  #[arg(long)]
  pub(crate) scene: Option<Scene>,
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
  #[arg(allow_hyphen_values = true, long)]
  pub(crate) vw: Option<f32>,
  #[arg(allow_hyphen_values = true, long)]
  pub(crate) vx: Option<f32>,
  #[arg(allow_hyphen_values = true, long)]
  pub(crate) vy: Option<f32>,
  #[arg(allow_hyphen_values = true, long)]
  pub(crate) vz: Option<f32>,
}

impl Options {
  pub(crate) fn state(&self) -> State {
    let mut state = if let Some(scene) = self.scene {
      scene.state()
    } else if let Some(program) = self.program {
      program.state()
    } else {
      default()
    };

    if let Some(db) = self.db {
      state.db = db;
    }

    if let Some(interpolate) = self.interpolate {
      state.interpolate = interpolate;
    }

    state.velocity = Vec4f::new(
      self.vx.unwrap_or(state.velocity.x),
      self.vy.unwrap_or(state.velocity.y),
      self.vz.unwrap_or(state.velocity.z),
      self.vw.unwrap_or(state.velocity.w),
    );

    state.fps = self.fps.or(state.fps);
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

  pub(crate) fn stream(&self, config: &Config) -> Result<Box<dyn Stream>> {
    if let Some(song) = &self.song {
      Ok(Box::new(Track::new(&config.find_song(song)?)?))
    } else if let Some(score) = self.score {
      Ok(Box::new(score.synthesizer()))
    } else if let Some(track) = &self.track {
      Ok(Box::new(Track::new(track)?))
    } else if let Some(program) = self.program {
      program.stream(config)
    } else {
      Ok(Box::new(Score::Silence.synthesizer()))
    }
  }
}
