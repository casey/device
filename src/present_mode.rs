use super::*;

#[derive(Clone, Copy, ValueEnum)]
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
