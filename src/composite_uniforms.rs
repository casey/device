use super::*;

#[derive(Default)]
pub(crate) struct CompositeUniforms {
  pub(crate) destination: bool,
  pub(crate) source: bool,
  pub(crate) viewport: Mat2x3f,
}

impl Uniforms for CompositeUniforms {
  fn write(&self, dst: &mut [u8]) -> usize {
    let mut i = 0;
    let mut a = Alignment::new(0);
    self.destination.write(dst, &mut i, &mut a);
    self.source.write(dst, &mut i, &mut a);
    self.viewport.write(dst, &mut i, &mut a);
    a.pad(i)
  }
}
