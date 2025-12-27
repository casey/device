use super::*;

#[derive(Clone, Copy, Debug, Default, EnumIter, IntoStaticStr)]
#[repr(u32)]
pub(crate) enum BlendMode {
  #[default]
  Destination,
  Source,
}

impl BlendMode {
  pub(crate) fn constant(self) -> String {
    format!("BLEND_MODE_{}", self.name().to_uppercase())
  }

  pub(crate) fn name(self) -> &'static str {
    self.into()
  }

  pub(crate) fn number(self) -> u32 {
    unsafe { *(&raw const self).cast() }
  }
}
