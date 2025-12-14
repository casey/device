use super::*;

#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(name(ModeKind))]
#[strum_discriminants(derive(Hash))]
pub(crate) enum Mode {
  Command(Vec<String>),
  Normal,
  Play,
}
