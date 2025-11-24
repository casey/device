use super::*;

mod capture;
mod probe;
mod run;
mod shader;

#[derive(Parser)]
pub(crate) enum Subcommand {
  Capture(capture::Capture),
  Probe,
  Run(run::Run),
  Shader,
}

impl Subcommand {
  pub(crate) fn run(self, options: Options) -> Result {
    match self {
      Self::Capture(capture) => capture.run(options),
      Self::Probe => probe::run(),
      Self::Shader => shader::run(),
      Self::Run(run) => run.run(options),
    }
  }
}

impl Default for Subcommand {
  fn default() -> Self {
    Self::Run(default())
  }
}
