use super::*;

#[derive(Clone, Copy)]
pub(crate) enum Command {
  App(fn(&mut App)),
  AppFallible(fn(&mut App) -> Result),
  State(fn(&mut State)),
}
