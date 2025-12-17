use super::*;

#[derive(Clone, Copy, Debug, ValueEnum, IntoStaticStr)]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum PresentMode {
  AutoVsync,
  Immediate,
}

impl From<PresentMode> for wgpu::PresentMode {
  fn from(present_mode: PresentMode) -> Self {
    match present_mode {
      PresentMode::AutoVsync => Self::AutoVsync,
      PresentMode::Immediate => Self::Immediate,
    }
  }
}

impl PresentMode {
  fn name(self) -> &'static str {
    self.into()
  }
}

impl Display for PresentMode {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    f.write_str(self.name())
  }
}
