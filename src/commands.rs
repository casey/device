use super::*;

pub(crate) struct Commands {
  map: BTreeMap<String, Command>,
}

impl Commands {
  pub(crate) fn complete(&self, prefix: &str) -> Option<&str> {
    self
      .map
      .range::<str, (Bound<&str>, Bound<&str>)>((Bound::Included(prefix), Bound::Unbounded))
      .next()
      .and_then(|(name, _command)| name.strip_prefix(prefix))
  }

  pub(crate) fn name(&self, s: &str) -> Option<Command> {
    self.map.get(s).copied()
  }

  pub(crate) fn new() -> Self {
    let mut map = BTreeMap::new();

    for (name, command) in generated::COMMANDS {
      map.insert(name.replace('_', "-"), *command);
    }

    Self { map }
  }
}

pub(crate) fn pop_command(app: &mut App) {
  app.pop_command();
}

pub(crate) fn complete_command(app: &mut App) {
  app.complete_command();
}

pub(crate) fn execute_command(app: &mut App, event_loop: &ActiveEventLoop) {
  app.execute_command(event_loop);
}

pub(crate) fn capture(app: &mut App) -> Result {
  app.capture()
}

pub(crate) fn enter_command_mode(app: &mut App) {
  app.enter_mode(Mode::Command(Vec::new()));
}

pub(crate) fn enter_play_mode(app: &mut App) {
  app.enter_mode(Mode::Play);
}

pub(crate) fn enter_normal_mode(app: &mut App) {
  app.enter_mode(Mode::Normal);
}

pub(crate) fn toggle_fullscreen(app: &mut App) {
  app.toggle_fullscreen();
}

pub(crate) fn set_patch_sine(app: &mut App) {
  app.set_patch(Patch::Sine);
}

pub(crate) fn set_patch_saw(app: &mut App) {
  app.set_patch(Patch::Saw);
}

pub(crate) fn negative_x_translation(state: &mut State) {
  state.filters.push(Filter {
    position: Mat3f::new_translation(&Vec2f::new(-0.1, 0.0)),
    wrap: state.wrap,
    ..default()
  });
}

pub(crate) fn positive_x_translation(state: &mut State) {
  state.filters.push(Filter {
    position: Mat3f::new_translation(&Vec2f::new(0.1, 0.0)),
    wrap: state.wrap,
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

pub(crate) fn clear_transient_x_translation(state: &mut State) {
  state.transient.x = 0.0;
}

pub(crate) fn clear_transient_y_translation(state: &mut State) {
  state.transient.y = 0.0;
}

pub(crate) fn clear_transient_scale(state: &mut State) {
  state.transient.z = 0.0;
}

pub(crate) fn blaster(state: &mut State) {
  state.filters = Scene::Blaster.state().filters;
}

pub(crate) fn bottom(state: &mut State) {
  state.filters.push(Filter {
    color: color::invert(),
    field: Field::Bottom,
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

pub(crate) fn cross(state: &mut State) {
  state.filters.push(Filter {
    color: color::invert(),
    field: Field::Cross,
    wrap: state.wrap,
    ..default()
  });
}

pub(crate) fn decrement_db(state: &mut State) {
  state.db -= 1.0;
}

pub(crate) fn frequencies(state: &mut State) {
  state.filters.push(Filter {
    color: color::invert(),
    field: Field::Frequencies,
    wrap: state.wrap,
    ..default()
  });
}

pub(crate) fn increment_db(state: &mut State) {
  state.db += 1.0;
}

pub(crate) fn left(state: &mut State) {
  state.filters.push(Filter {
    color: color::invert(),
    field: Field::Left,
    wrap: state.wrap,
    ..default()
  });
}

pub(crate) fn negative_rotation(state: &mut State) {
  state.filters.push(Filter {
    position: Mat3f::new_rotation(-0.01),
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

pub(crate) fn pop(state: &mut State) {
  state.pop();
}

pub(crate) fn positive_rotation(state: &mut State) {
  state.filters.push(Filter {
    position: Mat3f::new_rotation(0.01),
    ..default()
  });
}

pub(crate) fn reload_shaders(app: &mut App) {
  app.reload_shaders();
}

pub(crate) fn right(state: &mut State) {
  state.filters.push(Filter {
    color: color::invert(),
    field: Field::Right,
    wrap: state.wrap,
    ..default()
  });
}

pub(crate) fn samples(state: &mut State) {
  state.filters.push(Filter {
    color: color::invert(),
    field: Field::Samples,
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
  state.fit.toggle();
}

pub(crate) fn toggle_interpolate(state: &mut State) {
  state.interpolate.toggle();
}

pub(crate) fn toggle_repeat(state: &mut State) {
  state.filter.repeat.toggle();
}

pub(crate) fn toggle_tile(state: &mut State) {
  state.tile.toggle();
}

pub(crate) fn toggle_wrap(state: &mut State) {
  state.wrap.toggle();
}

pub(crate) fn top(state: &mut State) {
  state.filters.push(Filter {
    color: color::invert(),
    field: Field::Top,
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

pub(crate) fn x(state: &mut State) {
  state.filters.push(Filter {
    color: color::invert(),
    field: Field::X,
    wrap: state.wrap,
    ..default()
  });
}

pub(crate) fn zoom_in(state: &mut State) {
  state.filters.push(Filter {
    position: Mat3f::new_scaling(0.5),
    wrap: state.wrap,
    ..default()
  });
}

pub(crate) fn zoom_out(state: &mut State) {
  state.filters.push(Filter {
    position: Mat3f::new_scaling(2.0),
    wrap: state.wrap,
    ..default()
  });
}
