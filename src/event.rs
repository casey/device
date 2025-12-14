use super::*;

#[derive(Clone, Copy, Debug)]
pub(crate) enum Event {
  Button(Press),
  Encoder(u7),
}
