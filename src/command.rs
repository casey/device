use super::*;

pub(crate) enum Command {
  Nullary(fn(&mut State)),
}

impl From<fn(&mut State)> for Command {
  fn from(function: fn(&mut State)) -> Self {
    Self::Nullary(function)
  }
}

impl Command {
  pub(crate) fn call(&self, state: &mut State) {
    match self {
      Self::Nullary(function) => function(state),
    }
  }

  pub(crate) fn map() -> BTreeMap<&'static str, Command> {
    let mut map = BTreeMap::new();
    map.insert("left", Self::Nullary(left));
    map.insert("right", Self::Nullary(right));
    map.insert("spread", Self::Nullary(spread));
    map.insert("status", Self::Nullary(status));
    map
  }
}

fn left(state: &mut State) {
  state.filters.push(Filter {
    color: invert_color(),
    field: Field::Left,
    wrap: state.wrap,
    ..default()
  });
}

fn right(state: &mut State) {
  state.filters.push(Filter {
    color: invert_color(),
    field: Field::Right,
    wrap: state.wrap,
    ..default()
  });
}

fn spread(state: &mut State) {
  state.spread ^= true;
}

fn status(state: &mut State) {
  state.status ^= true;
}
