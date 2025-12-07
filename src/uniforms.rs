use super::*;

pub(crate) trait Uniforms: Default {
  fn size() -> u32 {
    let mut buffer = vec![0; MIB];
    Self::default().write(&mut buffer).try_into().unwrap()
  }

  fn write(&self, dst: &mut [u8]) -> usize;
}
