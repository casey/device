use super::*;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Ord, PartialOrd, IntoStaticStr)]
pub(crate) enum Controller {
  Spectra,
  Twister,
}

impl Controller {
  pub(crate) fn name(self) -> &'static str {
    self.into()
  }
}
