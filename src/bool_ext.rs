pub(crate) trait BoolExt {
  fn into_f32(self) -> f32;

  fn toggle(&mut self);
}

impl BoolExt for bool {
  fn into_f32(self) -> f32 {
    u8::from(self) as f32
  }

  fn toggle(&mut self) {
    *self = !*self;
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn into_f32() {
    assert_eq!(false.into_f32(), 0.0);
    assert_eq!(true.into_f32(), 1.0);
  }

  #[test]
  fn toggle() {
    let mut b = false;
    b.toggle();
    assert!(b);
    b.toggle();
    assert!(!b);
    b.toggle();
    assert!(b);
  }
}
