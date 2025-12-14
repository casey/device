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

pub(crate) fn advance(state: &mut State) {
  if state.beat % 4 == 3 {
    state.pop();
    state.pop();
  } else {
    push_top(state);
  }
  state.beat += 1;
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
  let presets = (0..Preset::LIMIT)
    .map(|i| Preset::random(&mut state.rng, i))
    .collect::<Vec<Preset>>();

  if log::log_enabled!(log::Level::Info) {
    let presets = presets
      .iter()
      .map(|preset| preset.name())
      .collect::<Vec<&str>>();
    log::info!("stack: {}", presets.join(" "));
  }

  state.truncate(0);

  state
    .filters
    .extend(presets.into_iter().map(Preset::filter));
}

pub(crate) fn bottom(state: &mut State) {
  state.filters.push(Filter {
    color: color::invert(),
    field: Field::Bottom,
    wrap: state.wrap,
    ..default()
  });
}

pub(crate) fn capture(app: &mut App) -> Result {
  let destination = app.config.capture("png");
  let tx = app.capture_tx.clone();
  app.renderer.as_ref().unwrap().capture(move |capture| {
    if let Err(err) = tx.send(capture.save(&destination)) {
      eprintln!("failed to send capture result: {err}");
    }
  })?;
  app.captures_pending += 1;
  Ok(())
}

pub(crate) fn circle(state: &mut State) {
  state.filters.push(Filter {
    color: color::invert(),
    field: Field::Circle { size: None },
    wrap: state.wrap,
    ..default()
  });
}

pub(crate) fn clear_transient_scale(state: &mut State) {
  state.transient.z = 0.0;
}

pub(crate) fn clear_transient_x_translation(state: &mut State) {
  state.transient.x = 0.0;
}

pub(crate) fn clear_transient_y_translation(state: &mut State) {
  state.transient.y = 0.0;
}

pub(crate) fn complete_command(app: &mut App) {
  let Mode::Command(command) = &mut app.mode else {
    return;
  };
  let prefix = command.iter().flat_map(|c| c.chars()).collect::<String>();
  if let Some(suffix) = app.commands.complete(&prefix) {
    if !suffix.is_empty() {
      eprintln!("completion: {prefix}{suffix}");
      command.push(suffix.into());
    }
  } else {
    eprintln!("no completion found for: {prefix}");
  }
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

pub(crate) fn cycle(state: &mut State) {
  if state.beat % 4 == 3 {
    state.pop();
    state.pop();
    state.pop();
  } else {
    push_top(state);
  }
  state.beat += 1;
}

pub(crate) fn cycle_zoom(state: &mut State) {
  if state.beat % 4 == 3 {
    state.pop();
    state.pop();
    state.pop();
  } else {
    zoom_out(state);
  }
  state.beat += 1;
}

pub(crate) fn decrement_db(state: &mut State) {
  state.db -= 1.0;
}

pub(crate) fn enter_command_mode(app: &mut App) {
  app.mode = Mode::Command(Vec::new());
}

pub(crate) fn enter_normal_mode(app: &mut App) {
  app.mode = Mode::Normal;
}

pub(crate) fn enter_play_mode(app: &mut App) {
  app.mode = Mode::Play;
}

pub(crate) fn execute_command(app: &mut App, event_loop: &ActiveEventLoop) {
  let Mode::Command(command) = &mut app.mode else {
    return;
  };
  let command = command.iter().flat_map(|c| c.chars()).collect::<String>();
  if let Some(command) = app.commands.name(command.as_str()) {
    app.dispatch(event_loop, command);
  } else {
    eprintln!("unknown command: {command}");
  }
  app.mode = Mode::Normal;
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

pub(crate) fn negative_x_translation(state: &mut State) {
  state.filters.push(Filter {
    position: Mat3f::new_translation(&Vec2f::new(-0.1, 0.0)),
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

pub(crate) fn pop(state: &mut State) {
  state.pop();
}

pub(crate) fn pop_command(app: &mut App) {
  let Mode::Command(command) = &mut app.mode else {
    return;
  };
  if command.pop().is_none() {
    app.mode = Mode::Normal;
  } else {
    app.print_command();
  }
}

pub(crate) fn positive_rotation(state: &mut State) {
  state.filters.push(Filter {
    position: Mat3f::new_rotation(0.01),
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

pub(crate) fn print(state: &mut State) {
  eprintln!(
    "{}",
    state
      .filters
      .iter()
      .map(|filter| filter.preset.map_or("unknown", Preset::name))
      .collect::<Vec<&str>>()
      .join(" ")
  );
}

pub(crate) fn push_bottom(state: &mut State) {
  state.filters.insert(
    0,
    Preset::random(&mut state.rng, state.filters.len()).filter(),
  );

  while state.filters.len() > Preset::LIMIT {
    state.filters.pop();
  }
}

pub(crate) fn push_top(state: &mut State) {
  state
    .filters
    .push(Preset::random(&mut state.rng, state.filters.len()).filter());

  while state.filters.len() > Preset::LIMIT {
    state.filters.remove(0);
  }
}

pub(crate) fn reload_shaders(app: &mut App) {
  if let Err(err) = app.renderer.as_mut().unwrap().reload_shaders() {
    eprintln!("failed to reload shader: {err}");
  }
}

pub(crate) fn right(state: &mut State) {
  state.filters.push(Filter {
    color: color::invert(),
    field: Field::Right,
    wrap: state.wrap,
    ..default()
  });
}

pub(crate) fn rotate_left(state: &mut State) {
  state.filters.rotate_left(1);
}

pub(crate) fn rotate_right(state: &mut State) {
  state.filters.rotate_right(1);
}

pub(crate) fn samples(state: &mut State) {
  state.filters.push(Filter {
    color: color::invert(),
    field: Field::Samples,
    wrap: state.wrap,
    ..default()
  });
}

pub(crate) fn set_patch_saw(app: &mut App) {
  app.patch = Patch::Saw;
}

pub(crate) fn set_patch_sine(app: &mut App) {
  app.patch = Patch::Sine;
}

pub(crate) fn shuffle(state: &mut State) {
  state.filters.shuffle(&mut state.rng);
}

pub(crate) fn spread(state: &mut State) {
  state.spread.toggle();
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
  state.status.toggle();
}

pub(crate) fn swap(state: &mut State) {
  if state.filters.len() > 2 {
    let a = state.filters.pop().unwrap();
    let b = state.filters.pop().unwrap();
    state.filters.push(a);
    state.filters.push(b);
  }
}

pub(crate) fn toggle_fit(state: &mut State) {
  state.fit.toggle();
}

pub(crate) fn toggle_fullscreen(app: &mut App) {
  app.fullscreen.toggle();
  app
    .window()
    .set_fullscreen(app.fullscreen.then_some(Fullscreen::Borderless(None)));
}

pub(crate) fn toggle_interpolate(state: &mut State) {
  state.interpolate.toggle();
}

pub(crate) fn toggle_muted(app: &mut App) {
  app.tap.toggle_muted();
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

pub(crate) fn undo(app: &mut App) {
  if let Some(state) = app.history.pop() {
    app.state = state;
  }
}

pub(crate) fn unwind(app: &mut App) {
  app.unwind = true;
}

pub(crate) fn waffle(app: &mut App) {
  if let Some(mut state) = app.history.pop() {
    mem::swap(&mut state, &mut app.state);
    app.history.push(state);
  }
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
