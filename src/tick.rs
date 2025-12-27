use super::*;

#[derive(Clone, Copy)]
pub(crate) struct Tick {
  pub(crate) dt: Duration,
  pub(crate) last: Option<Position>,
  pub(crate) position: Option<Position>,
  pub(crate) tempo: Option<Tempo>,
  pub(crate) time: f64,
}

impl Tick {
  pub(crate) fn advance(self) -> Option<Position> {
    if self.position == self.last {
      None
    } else {
      self.position
    }
  }

  pub(crate) fn advanced(self) -> bool {
    self.advance().is_some()
  }
}
