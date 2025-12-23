use super::*;

#[derive(Clone, Copy, Debug)]
pub(crate) enum Command {
  App(fn(&mut App)),
  AppEventLoop(fn(&mut App, &ActiveEventLoop)),
  AppFallible(fn(&mut App) -> Result),
  History(fn(&mut History)),
  HistoryState(fn(&mut History, &mut State)),
  RngState(fn(&mut SmallRng, &mut State)),
  State(fn(&mut State)),
}
