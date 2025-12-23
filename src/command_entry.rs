use super::*;

#[derive(Clone, Copy, Debug)]
pub(crate) struct CommandEntry {
  pub(crate) command: Command,
  pub(crate) name: &'static str,
}

impl CommandEntry {
  pub(crate) const fn new(name: &'static str, command: Command) -> Self {
    Self { command, name }
  }
}
