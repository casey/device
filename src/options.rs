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
  #[arg(long)]
  pub(crate) fps: Option<Fps>,
  #[arg(group = AUDIO, long)]
  pub(crate) input: bool,
  #[arg(long)]
  pub(crate) program: Option<Program>,
  #[arg(long)]
  pub(crate) record: bool,
  #[arg(long)]
  pub(crate) resolution: Option<NonZeroU32>,
  #[arg(group = AUDIO, long)]
  pub(crate) song: Option<String>,
  #[arg(group = AUDIO, long)]
  pub(crate) synthesizer: bool,
  #[arg(group = AUDIO, long)]
  pub(crate) track: Option<Utf8PathBuf>,
  #[arg(long)]
  pub(crate) verbose: bool,
  #[arg(long)]
  pub(crate) volume: Option<f32>,
}

impl Options {
  pub(crate) fn state(&self) -> State {
    let mut state = self.program.map(Program::state).unwrap_or_default();
    state.fps = self.fps.or(state.fps);
    state.resolution = self.resolution.or(state.resolution);
    state
  }
}
