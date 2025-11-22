use super::*;

mod capture;
mod probe;
mod run;
mod shader;

#[derive(Default, Parser)]
pub(crate) enum Subcommand {
  Capture(capture::Capture),
  Probe,
  #[default]
  Run,
  Shader,
}

impl Subcommand {
  pub(crate) fn run(self, options: Options) -> Result {
    match self {
      Self::Capture(capture) => capture.run(options),
      Self::Probe => probe::run(),
      Self::Shader => shader::run(),
      Self::Run => run::run(options),
    }
  }
}
