use super::*;

#[derive(Default)]
pub(crate) struct FilterUniforms {
  pub(crate) alpha: f32,
  pub(crate) base: f32,
  pub(crate) color: Mat3x4f,
  pub(crate) coordinates: bool,
  pub(crate) field: Field,
  pub(crate) frequency_range: f32,
  pub(crate) front_offset: Vec2f,
  pub(crate) gain: f32,
  pub(crate) grid: f32,
  pub(crate) grid_alpha: f32,
  pub(crate) interpolate: bool,
  pub(crate) mirror: Vec4f,
  pub(crate) offset: Vec2f,
  pub(crate) parameter: f32,
  pub(crate) position: Mat2x3f,
  pub(crate) repeat: bool,
  pub(crate) resolution: f32,
  pub(crate) response: f32,
  pub(crate) sample_range: f32,
  pub(crate) tiling: u32,
  pub(crate) wrap: bool,
}

impl Uniforms for FilterUniforms {
  fn write(&self, dst: &mut [u8]) -> usize {
    let mut i = 0;
    let mut a = Alignment::new(1);
    self.alpha.write(dst, &mut i, &mut a);
    self.base.write(dst, &mut i, &mut a);
    self.color.write(dst, &mut i, &mut a);
    self.coordinates.write(dst, &mut i, &mut a);
    self.field.write(dst, &mut i, &mut a);
    self.frequency_range.write(dst, &mut i, &mut a);
    self.front_offset.write(dst, &mut i, &mut a);
    self.gain.write(dst, &mut i, &mut a);
    self.grid.write(dst, &mut i, &mut a);
    self.grid_alpha.write(dst, &mut i, &mut a);
    self.interpolate.write(dst, &mut i, &mut a);
    self.mirror.write(dst, &mut i, &mut a);
    self.offset.write(dst, &mut i, &mut a);
    self.parameter.write(dst, &mut i, &mut a);
    self.position.write(dst, &mut i, &mut a);
    self.repeat.write(dst, &mut i, &mut a);
    self.resolution.write(dst, &mut i, &mut a);
    self.response.write(dst, &mut i, &mut a);
    self.sample_range.write(dst, &mut i, &mut a);
    self.tiling.write(dst, &mut i, &mut a);
    self.wrap.write(dst, &mut i, &mut a);
    a.pad(i)
  }
}
