use {
  super::*,
  clap::{
    ArgGroup,
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
  #[arg(group = AUDIO, long)]
  pub(crate) input: bool,
  #[arg(long)]
  pub(crate) program: Option<Program>,
  #[arg(long)]
  pub(crate) record: bool,
  #[arg(group = AUDIO, long)]
  pub(crate) song: Option<String>,
  #[arg(group = AUDIO, long)]
  pub(crate) synthesizer: bool,
  #[arg(group = AUDIO, long)]
  pub(crate) track: Option<Utf8PathBuf>,
  #[arg(long)]
  pub(crate) volume: Option<f32>,
}
