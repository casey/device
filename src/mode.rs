use super::*;

#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(name(ModeKind))]
#[strum_discriminants(derive(Hash, Ord, PartialOrd, IntoStaticStr))]
pub(crate) enum Mode {
  Command(Vec<String>),
  Normal,
  Play,
}

impl ModeKind {
  pub(crate) fn name(self) -> &'static str {
    self.into()
  }
}
