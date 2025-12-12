#[derive(Default)]
pub(crate) struct Counter(u32);

impl Counter {
  pub(crate) fn next(&mut self) -> u32 {
    let next = self.0;
    self.0 += 1;
    next
  }
}
