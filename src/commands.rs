use super::*;

pub(crate) struct Commands {
  map: BTreeMap<String, fn(&mut State)>,
}

impl Commands {
  pub(crate) fn new() -> Self {
    let mut map = BTreeMap::new();

    for (name, command) in COMMANDS {
      map.insert(name.replace('_', "-"), *command);
    }

    Self { map }
  }

  pub(crate) fn name(&self, s: &str) -> Option<fn(&mut State)> {
    self.map.get(s).copied()
  }

  pub(crate) fn complete(&self, prefix: &str) -> Option<&str> {
    self
      .map
      .range::<str, (Bound<&str>, Bound<&str>)>((Bound::Included(prefix), Bound::Unbounded))
      .next()
      .and_then(|(name, _command)| name.strip_prefix(&prefix))
  }
}

macro_rules! commands {
  {
    $($command:ident)*
  } => {
    const COMMANDS: &[(&'static str, fn(&mut State))] = &[
      $(
        (stringify!($command), $command),
      )*
    ];
  }
}

commands! {
  all
  blaster
  bottom
  circle
  coordinates
  cross
  decrement_db
  frequencies
  increment_db
  left
  negative_rotation
  none
  pop
  positive_rotation
  right
  samples
  spread
  square
  status
  toggle_fit
  toggle_interpolate
  toggle_repeat
  toggle_tile
  toggle_wrap
  top
  triangle
  x
  zoom
}

pub(crate) fn pop(state: &mut State) {
  state.pop();
}

pub(crate) fn negative_rotation(state: &mut State) {
  state.filters.push(Filter {
    position: Mat3f::new_rotation(-0.01),
    ..default()
  });
}

pub(crate) fn positive_rotation(state: &mut State) {
  state.filters.push(Filter {
    position: Mat3f::new_rotation(0.01),
    ..default()
  });
}

pub(crate) fn all(state: &mut State) {
  state.filters.push(Filter {
    color: color::invert(),
    field: Field::All,
    wrap: state.wrap,
    ..default()
  });
}

pub(crate) fn blaster(state: &mut State) {
  state.filters = Scene::Blaster.state().filters;
}

pub(crate) fn cross(state: &mut State) {
  state.filters.push(Filter {
    color: color::invert(),
    field: Field::Cross,
    wrap: state.wrap,
    ..default()
  });
}

pub(crate) fn increment_db(state: &mut State) {
  state.db += 1.0;
}

pub(crate) fn decrement_db(state: &mut State) {
  state.db -= 1.0;
}

pub(crate) fn left(state: &mut State) {
  state.filters.push(Filter {
    color: color::invert(),
    field: Field::Left,
    wrap: state.wrap,
    ..default()
  });
}

pub(crate) fn right(state: &mut State) {
  state.filters.push(Filter {
    color: color::invert(),
    field: Field::Right,
    wrap: state.wrap,
    ..default()
  });
}

pub(crate) fn spread(state: &mut State) {
  state.spread ^= true;
}

pub(crate) fn square(state: &mut State) {
  state.filters.push(Filter {
    color: color::invert(),
    field: Field::Square,
    wrap: state.wrap,
    ..default()
  });
}

pub(crate) fn status(state: &mut State) {
  state.status ^= true;
}

pub(crate) fn toggle_fit(state: &mut State) {
  state.fit.toggle()
}

pub(crate) fn top(state: &mut State) {
  state.filters.push(Filter {
    color: color::invert(),
    field: Field::Top,
    wrap: state.wrap,
    ..default()
  });
}

pub(crate) fn bottom(state: &mut State) {
  state.filters.push(Filter {
    color: color::invert(),
    field: Field::Bottom,
    wrap: state.wrap,
    ..default()
  });
}

pub(crate) fn triangle(state: &mut State) {
  state.filters.push(Filter {
    color: color::invert(),
    field: Field::Triangle,
    wrap: state.wrap,
    ..default()
  });
}

pub(crate) fn circle(state: &mut State) {
  state.filters.push(Filter {
    color: color::invert(),
    field: Field::Circle { size: None },
    wrap: state.wrap,
    ..default()
  });
}

pub(crate) fn coordinates(state: &mut State) {
  state.filters.push(Filter {
    coordinates: true,
    wrap: state.wrap,
    ..default()
  });
}

pub(crate) fn toggle_interpolate(state: &mut State) {
  state.interpolate.toggle();
}

pub(crate) fn frequencies(state: &mut State) {
  state.filters.push(Filter {
    color: color::invert(),
    field: Field::Frequencies,
    wrap: state.wrap,
    ..default()
  });
}

pub(crate) fn none(state: &mut State) {
  state.filters.push(Filter {
    field: Field::None,
    wrap: state.wrap,
    ..default()
  });
}

pub(crate) fn toggle_repeat(state: &mut State) {
  state.filter.repeat.toggle();
}

pub(crate) fn samples(state: &mut State) {
  state.filters.push(Filter {
    color: color::invert(),
    field: Field::Samples,
    wrap: state.wrap,
    ..default()
  })
}

pub(crate) fn toggle_tile(state: &mut State) {
  state.tile.toggle();
}

pub(crate) fn toggle_wrap(state: &mut State) {
  state.wrap.toggle();
}

pub(crate) fn x(state: &mut State) {
  state.filters.push(Filter {
    color: color::invert(),
    field: Field::X,
    wrap: state.wrap,
    ..default()
  });
}

pub(crate) fn zoom(state: &mut State) {
  state.filters.push(Filter {
    position: Mat3f::new_scaling(2.0),
    wrap: state.wrap,
    ..default()
  });
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn commands_are_lowercase() {
    for (name, _command) in COMMANDS {
      assert_eq!(name.to_lowercase(), *name);
    }
  }

  #[test]
  fn commands_are_unique() {
    let mut names = HashSet::new();
    for (name, _command) in COMMANDS {
      assert!(names.insert(name), "duplicate command: {name}");
    }
  }

  #[test]
  fn all_commands_are_mapped() {
    let commands = Commands::new();

    let s = include_str!("commands.rs");

    for m in Regex::new(r"(?m)^(?:pub\(crate\)\s*)?fn (.*?)\(")
      .unwrap()
      .captures_iter(s)
    {
      let name = m[1].replace('_', "-");
      assert!(
        commands.map.contains_key(&name),
        "unmapped commmand: {name}",
      );
    }
  }
}
