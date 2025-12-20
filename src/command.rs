use super::*;

#[derive(Clone, Copy, Debug)]
pub(crate) enum Command {
  App(fn(&mut App)),
  AppEventLoop(fn(&mut App, &ActiveEventLoop)),
  AppFallible(fn(&mut App) -> Result),
  State(fn(&mut State)),
}
