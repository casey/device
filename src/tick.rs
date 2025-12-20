use super::*;

#[derive(Clone, Copy)]
pub(crate) struct Tick {
  pub(crate) advance: bool,
  pub(crate) beat: Option<u64>,
  pub(crate) dt: Duration,
}
