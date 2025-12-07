use super::*;

#[derive(Default)]
pub(crate) struct CompositeUniforms {
  pub(crate) back_read: bool,
  pub(crate) fit: bool,
  pub(crate) front_read: bool,
  pub(crate) resolution: Vec2f,
}

impl Uniforms for CompositeUniforms {
  fn write(&self, dst: &mut [u8]) -> usize {
    let mut i = 0;
    let mut a = 0;
    self.back_read.write(dst, &mut i, &mut a);
    self.fit.write(dst, &mut i, &mut a);
    self.front_read.write(dst, &mut i, &mut a);
    self.resolution.write(dst, &mut i, &mut a);
    pad(i, a)
  }
}
