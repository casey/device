use super::*;

#[derive(Default)]
pub(crate) struct History {
  pub(crate) commands: Vec<Command>,
  pub(crate) states: Vec<State>,
  pub(crate) unwind: bool,
}

impl History {
  pub(crate) fn tick(&mut self, current: &mut State) {
    if !self.unwind {
      return;
    }

    let Some(last) = self.states.pop() else {
      self.unwind = false;
      return;
    };

    *current = last;
  }
}
