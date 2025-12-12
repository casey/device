#[derive(Default)]
pub(crate) struct Counter(u32);

impl Counter {
  pub(crate) fn next(&mut self) -> u32 {
    let next = self.0;
    self.0 += 1;
    next
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn counter() {
    let mut c = Counter::default();
    assert_eq!(c.next(), 0);
    assert_eq!(c.next(), 1);
    assert_eq!(c.next(), 2);
  }
}
