use super::*;

#[derive(
  Clone, Copy, Debug, Eq, Hash, PartialEq, Ord, PartialOrd, strum::Display, IntoStaticStr,
)]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum Controller {
  Spectra,
  Twister,
}

impl Controller {
  pub(crate) fn name(self) -> &'static str {
    self.into()
  }
}
