use super::*;

#[derive(Clone, Copy, Debug)]
pub(crate) struct CommandEntry {
  pub(crate) name: &'static str,
  pub(crate) command: Command,
}

impl CommandEntry {
  pub(crate) const fn new(name: &'static str, command: Command) -> Self {
    Self { name, command }
  }
}
