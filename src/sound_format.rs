use super::*;

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct SoundFormat {
  pub(crate) channels: u16,
  pub(crate) sample_rate: u32,
}

impl Display for SoundFormat {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}x{}", self.channels, self.sample_rate)
  }
}
