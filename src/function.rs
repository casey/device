use super::*;

pub(crate) enum Function {
  Nullary(fn(&mut State)),
}

impl From<fn(&mut State)> for Function {
  fn from(function: fn(&mut State)) -> Self {
    Function::Nullary(function)
  }
}
// match command.as_str() {
//   "left" => self.state.filters.push(Filter {
//     color: invert_color(),
//     field: Field::Left,
//     wrap: self.state.wrap,
//     ..default()
//   }),
//   "right" => self.state.filters.push(Filter {
//     color: invert_color(),
//     field: Field::Right,
//     wrap: self.state.wrap,
//     ..default()
//   }),
//   "spread" => self.state.spread = !self.state.spread,
//   "status" => self.state.status = !self.state.status,
//   _ => eprintln!("unknown command: {command}"),
// }
// self.command = None;

impl Function {
  pub(crate) fn call(&self, state: &mut State) {
    match self {
      Self::Nullary(function) => function(state),
    }
  }

  pub(crate) fn map() -> BTreeMap<&'static str, Function> {
    let mut map = BTreeMap::new();
    map.insert("left", Function::Nullary(left));
    map.insert("right", Function::Nullary(right));
    map.insert("spread", Function::Nullary(spread));
    map.insert("status", Function::Nullary(status));
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
    field: Field::Left,
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
