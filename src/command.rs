use super::*;

#[derive(Clone, Copy)]
pub(crate) enum Command {
  App(fn(&mut App)),
  State(fn(&mut State)),
}
