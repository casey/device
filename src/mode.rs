use super::*;

#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(name(ModeKind))]
#[strum_discriminants(derive(Hash, Ord, PartialOrd, IntoStaticStr))]
#[strum_discriminants(allow(clippy::arbitrary_source_item_ordering))]
#[allow(clippy::arbitrary_source_item_ordering)]
pub(crate) enum Mode {
  Normal,
  Command(Vec<String>),
  Play,
}

impl ModeKind {
  pub(crate) fn name(self) -> &'static str {
    self.into()
  }
}
