use super::*;

#[derive(Clone, Copy)]
pub(crate) struct Tick {
  pub(crate) dt: Duration,
  pub(crate) last: Option<Position>,
  pub(crate) position: Option<Position>,
}

impl Tick {
  pub(crate) fn advance(self) -> bool {
    self.position != self.last
  }
}
