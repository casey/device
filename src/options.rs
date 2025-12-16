use {
  super::*,
  clap::{
    ArgAction,
    builder::styling::{AnsiColor, Effects, Styles},
  },
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
  #[arg(long, action = ArgAction::SetTrue)]
  pub(crate) fit: Option<bool>,
  #[arg(long)]
  pub(crate) format: Option<ImageFormat>,
  #[arg(long)]
  pub(crate) fps: Option<Fps>,
  #[arg(long)]
  pub(crate) height: Option<NonZeroU32>,
  #[arg(group = AUDIO, long)]
  pub(crate) input: bool,
  #[arg(long)]
  pub(crate) interpolate: Option<bool>,
  #[arg(long)]
  pub(crate) mute: bool,
  #[arg(long)]
  pub(crate) preset: Option<Vec<Preset>>,
  #[arg(long)]
  pub(crate) program: Option<Program>,
  #[arg(long)]
  pub(crate) resolution: Option<NonZeroU32>,
  #[arg(long)]
  pub(crate) scene: Option<Scene>,
  #[arg(group = AUDIO, long)]
  pub(crate) score: Option<Score>,
  #[arg(long)]
  pub(crate) seed: Option<u64>,
  #[arg(group = AUDIO, long)]
  pub(crate) song: Option<String>,
  #[arg(group = AUDIO, long)]
  pub(crate) track: Option<Utf8PathBuf>,
  #[arg(long)]
  pub(crate) verbose: bool,
  #[arg(allow_hyphen_values = true, long)]
  pub(crate) vw: Option<f32>,
  #[arg(allow_hyphen_values = true, long)]
  pub(crate) vx: Option<f32>,
  #[arg(allow_hyphen_values = true, long)]
  pub(crate) vy: Option<f32>,
  #[arg(allow_hyphen_values = true, long)]
  pub(crate) vz: Option<f32>,
  #[arg(long)]
  pub(crate) width: Option<NonZeroU32>,
}

impl Options {
  pub(crate) fn add_source(&self, config: &Config, tap: &mut Tap) -> Result {
    if let Some(song) = &self.song {
      let wave = tap.load_wave(&config.find_song(song)?)?;
      tap.sequence_wave(&wave, 0.0, 0.0);
    } else if let Some(score) = self.score {
      score.sequence(tap);
    } else if let Some(track) = &self.track {
      let wave = tap.load_wave(track)?;
      tap.sequence_wave(&wave, 0.0, 0.0);
    } else if let Some(program) = self.program {
      program.add_source(config, tap)?;
    }

    Ok(())
  }

  pub(crate) fn image_format(&self) -> Option<ImageFormat> {
    self
      .format
      .or_else(|| self.scene.and_then(Scene::format))
      .or_else(|| self.program.and_then(|program| program.scene().format()))
  }

  pub(crate) fn size(&self, size: Vector2<NonZeroU32>) -> (Vector2<NonZeroU32>, NonZeroU32) {
    let size = Vector2::new(self.width.unwrap_or(size.x), self.height.unwrap_or(size.y));
    let resolution = self.resolution.unwrap_or(size.x.max(size.y));
    (size, resolution)
  }

  pub(crate) fn state(&self) -> State {
    let mut state = if let Some(scene) = self.scene {
      scene.state(self.seed)
    } else if let Some(program) = self.program {
      let mut state = program.scene().state(self.seed);
      if let Some(db) = program.db() {
        state.db = db;
      }
      state
    } else {
      default()
    };

    if let Some(presets) = &self.preset {
      for preset in presets {
        state.filters.push(preset.filter());
      }
    }

    state.db = self.db.unwrap_or(state.db);
    state.fit = self.fit.unwrap_or(state.fit);
    state.interpolate = self.interpolate.unwrap_or(state.interpolate);
    state.velocity = Vec4f::new(
      self.vx.unwrap_or(state.velocity.x),
      self.vy.unwrap_or(state.velocity.y),
      self.vz.unwrap_or(state.velocity.z),
      self.vw.unwrap_or(state.velocity.w),
    );

    state
  }

  pub(crate) fn stdio(&self) -> Stdio {
    if self.verbose {
      Stdio::inherit()
    } else {
      Stdio::piped()
    }
  }
}
