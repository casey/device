use super::*;

mod bindings;
mod capture;
mod probe;
mod run;
mod shader;

#[derive(Parser)]
pub(crate) enum Subcommand {
  Bindings,
  Capture(capture::Capture),
  Probe,
  Run(run::Run),
  Shader,
}

impl Subcommand {
  pub(crate) fn run(self, options: Options, config: Config) -> Result {
    match self {
      Self::Bindings => bindings::run(),
      Self::Capture(capture) => capture.run(options, config),
      Self::Probe => probe::run(),
      Self::Run(run) => run.run(options, config),
      Self::Shader => shader::run(),
    }
  }
}

impl Default for Subcommand {
  fn default() -> Self {
    Self::Run(default())
  }
}
