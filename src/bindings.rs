use super::*;

#[rustfmt::skip]
const BUTTON_BINDINGS: &[(Controller, u8, Press, (&str, Command))] = {
  use {Controller::*, generated::*, Press::Press};
  &[
    (Spectra,  0, Press, PUSH_TOP),
    (Spectra,  1, Press, PUSH_BOTTOM),
    (Spectra,  2, Press, CYCLE),
    (Spectra,  3, Press, CYCLE_ZOOM),

    (Spectra,  4, Press, ROTATE_LEFT),
    (Spectra,  5, Press, ROTATE_RIGHT),
    (Spectra,  6, Press, SHUFFLE),
    (Spectra,  7, Press, SWAP),

    (Spectra,  8, Press, ADVANCE),
    (Spectra,  9, Press, BLASTER),
    (Spectra, 10, Press, WAFFLE),

    (Twister,  4, Press, CLEAR_TRANSIENT_X_TRANSLATION),
    (Twister,  5, Press, CLEAR_TRANSIENT_Y_TRANSLATION),
    (Twister,  6, Press, CLEAR_TRANSIENT_SCALE),
  ]
};

#[rustfmt::skip]
const CHARACTER_BINDINGS: &[(ModeKind, char, ModifiersState, (&str, Command))] = {
  use {generated::*, ModeKind::*};

  const OFF: ModifiersState = ModifiersState::empty();
  const CTRL: ModifiersState = ModifiersState::CONTROL;
  const CTRL_SUPER: ModifiersState = CTRL.union(SUPER);
  const SHIFT: ModifiersState = ModifiersState::SHIFT;
  const SUPER: ModifiersState = ModifiersState::SUPER;

  &[
    (Normal, '+',  OFF,        INCREMENT_DB),
    (Normal, '-',  OFF,        DECREMENT_DB),
    (Normal, ':',  OFF,        ENTER_COMMAND_MODE),
    (Normal, '>',  OFF,        CAPTURE),
    (Normal, '?',  OFF,        PRINT),
    (Normal, 'A',  OFF,        ALL),
    (Normal, 'B',  OFF,        BLASTER),
    (Normal, 'C',  OFF,        CIRCLE),
    (Normal, 'D',  OFF,        COORDINATES),
    (Normal, 'E',  OFF,        CLEAR_ELAPSED),
    (Normal, 'F',  CTRL_SUPER, TOGGLE_FULLSCREEN),
    (Normal, 'F',  OFF,        TOGGLE_FIT),
    (Normal, 'I',  OFF,        TOGGLE_INTERPOLATE),
    (Normal, 'L',  OFF,        FREQUENCIES),
    (Normal, 'N',  OFF,        NONE),
    (Normal, 'P',  OFF,        ENTER_PLAY_MODE),
    (Normal, 'R',  OFF,        TOGGLE_REPEAT),
    (Normal, 'R',  SHIFT,      RELOAD_SHADERS),
    (Normal, 'S',  OFF,        SAMPLES),
    (Normal, 'T',  OFF,        TOGGLE_TILE),
    (Normal, 'U',  SUPER,      UNWIND),
    (Normal, 'W',  OFF,        TOGGLE_WRAP),
    (Normal, 'X',  OFF,        X),
    (Normal, 'Z',  OFF,        ZOOM_OUT),
    (Normal, 'Z',  SUPER,      UNDO),
    (Normal, '\\', SUPER,     TOGGLE_MUTED),
    (Play,   '1',  OFF,        SET_PATCH_SINE),
    (Play,   '2',  OFF,        SET_PATCH_SAW),
  ]
};

const ENCODER_BINDINGS: &[(Controller, u8, fn(&mut State, u7) -> f32)] = {
  use Controller::*;

  fn integer(value: u7) -> f32 {
    let value = i8::try_from(u8::from(value)).unwrap() - 64;
    if value == -1 { 0.0 } else { value as f32 }
  }

  fn float(value: u7) -> f32 {
    let value = integer(value);
    if value < 0.0 {
      value / 64.0
    } else {
      value / 63.0
    }
  }

  fn set_alpha(state: &mut State, value: u7) -> f32 {
    let value = float(value).midpoint(1.0);
    state.alpha = value;
    value
  }

  fn set_complexity(state: &mut State, value: u7) -> f32 {
    let value = float(value);
    state.complexity = value;
    value
  }

  fn set_db(state: &mut State, value: u7) -> f32 {
    let value = integer(value);
    state.db = value;
    value
  }

  fn set_velocity_x(state: &mut State, value: u7) -> f32 {
    let value = float(value);
    state.velocity.x = value;
    value
  }

  fn set_velocity_y(state: &mut State, value: u7) -> f32 {
    let value = float(value);
    state.velocity.y = value;
    value
  }

  fn set_velocity_scaling(state: &mut State, value: u7) -> f32 {
    let value = float(value);
    state.velocity.z = value;
    value
  }

  fn set_velocity_rotation(state: &mut State, value: u7) -> f32 {
    let value = float(value);
    state.velocity.w = value;
    value
  }

  &[
    (Twister, 0, set_alpha),
    (Twister, 1, set_db),
    (Twister, 4, set_velocity_x),
    (Twister, 5, set_velocity_y),
    (Twister, 6, set_velocity_scaling),
    (Twister, 7, set_velocity_rotation),
    (Twister, 8, set_complexity),
  ]
};

#[rustfmt::skip]
const NAMED_BINDINGS: &[(ModeKind, NamedKey, ModifiersState, (&str, Command))] = {
  use {
    ModeKind::{Normal, Play, Command},
    NamedKey::*,
    generated::*,
  };

  const OFF: ModifiersState = ModifiersState::empty();

  &[
    (Command, Backspace,  OFF, POP_COMMAND),
    (Command, Enter,      OFF, EXECUTE_COMMAND),
    (Command, Escape,     OFF, ENTER_NORMAL_MODE),
    (Command, Tab,        OFF, COMPLETE_COMMAND),
    (Normal,  ArrowLeft,  OFF, NEGATIVE_ROTATION),
    (Normal,  ArrowRight, OFF, POSITIVE_ROTATION),
    (Normal,  Backspace,  OFF, POP),
    (Play,    Escape,     OFF, ENTER_NORMAL_MODE),
  ]
};

pub(crate) struct Bindings {
  button: BTreeMap<(Controller, u8, Press), (&'static str, Command)>,
  character: BTreeMap<(ModeKind, String, ModifiersState), (&'static str, Command)>,
  encoder: BTreeMap<(Controller, u8), fn(&mut State, u7) -> f32>,
  named: BTreeMap<(ModeKind, NamedKey, ModifiersState), (&'static str, Command)>,
}

impl Bindings {
  pub(crate) fn button(&self, controller: Controller, button: u8, press: Press) -> Option<Command> {
    let command = self.button.get(&(controller, button, press)).copied();

    if command.is_none() {
      log::info!("unbound button: {controller:?} {button} {press:?}");
    }

    command.map(|command| command.1)
  }

  pub(crate) fn encoder(
    &self,
    controller: Controller,
    encoder: u8,
  ) -> Option<fn(&mut State, u7) -> f32> {
    let command = self.encoder.get(&(controller, encoder)).copied();

    if command.is_none() {
      log::info!("unbound encoder: {controller:?} {encoder}");
    }

    command
  }

  pub(crate) fn key(&self, mode: ModeKind, key: &Key, modifiers: Modifiers) -> Option<Command> {
    let command = match key {
      Key::Character(character) => {
        let character = character.to_uppercase();

        let mut modifiers = modifiers.state();
        if character == character.to_lowercase() {
          modifiers.remove(ModifiersState::SHIFT);
        }

        self.character.get(&(mode, character, modifiers)).copied()
      }
      Key::Named(named) => self.named.get(&(mode, *named, modifiers.state())).copied(),
      _ => None,
    };

    if command.is_none() {
      log::info!("unbound key: {key:?} {modifiers:?}");
    }

    command.map(|command| command.1)
  }

  pub(crate) fn new() -> Self {
    Self {
      button: BUTTON_BINDINGS
        .iter()
        .copied()
        .map(|(controller, control, pressed, command)| ((controller, control, pressed), command))
        .collect(),
      character: CHARACTER_BINDINGS
        .iter()
        .map(|(mode, character, modifiers, command)| {
          ((*mode, character.to_string(), *modifiers), *command)
        })
        .collect(),
      encoder: ENCODER_BINDINGS
        .iter()
        .copied()
        .map(|(controller, control, command)| ((controller, control), command))
        .collect(),
      named: NAMED_BINDINGS
        .iter()
        .copied()
        .map(|(mode, named, modifiers, command)| ((mode, named, modifiers), command))
        .collect(),
    }
  }
}

impl Display for Bindings {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    use tabled::{
      builder::Builder,
      settings::{Alignment, Panel, Span, Style, object::Columns, themes::BorderCorrection},
    };

    fn binding(modifiers: ModifiersState, key: &str) -> String {
      let mut binding = Vec::new();

      if modifiers.control_key() {
        binding.push("⌃");
      }

      if modifiers.alt_key() {
        binding.push("⌥");
      }

      if modifiers.shift_key() {
        binding.push("⇧");
      }

      if modifiers.super_key() {
        binding.push("⌘");
      }

      binding.push(key);

      binding.join(" ")
    }

    let mut modes = BTreeMap::<ModeKind, Vec<[String; 2]>>::new();

    for ((mode, character, modifiers), (name, _command)) in &self.character {
      modes
        .entry(*mode)
        .or_default()
        .push([binding(*modifiers, character), (*name).into()]);
    }

    for ((mode, named_key, modifiers), (name, _command)) in &self.named {
      modes.entry(*mode).or_default().push([
        binding(*modifiers, &format!("{named_key:?}")),
        (*name).into(),
      ]);
    }

    let mut builder = Builder::default();

    let mut mode_records = Vec::new();

    for (mode, records) in modes {
      mode_records.push(builder.count_records());
      builder.push_record([mode.name()]);

      for record in records {
        builder.push_record(record);
      }
    }

    let mut table = builder.build();
    table.modify(Columns::first(), Alignment::right());
    for i in mode_records {
      table.modify((i, 0), Span::column(2));
      table.modify((i, 0), Alignment::center());
    }

    writeln!(
      f,
      "{}",
      table.with(Style::modern()).with(BorderCorrection::span()),
    )?;

    let mut spectra = [[""; 4]; 4];
    for ((controller, control, _press), (name, _command)) in &self.button {
      if *controller == Controller::Spectra {
        let i = control.into_usize();
        if i >= 16 {
          continue;
        }
        spectra[i / 4][i % 4] = name;
      }
    }

    let mut builder = Builder::default();
    for row in spectra {
      builder.push_record(row);
    }

    writeln!(
      f,
      "{}",
      builder
        .build()
        .with(Style::modern())
        .with(Panel::header(Controller::Spectra.name()))
        .with(BorderCorrection::span())
    )?;

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn button_bindings_are_unique() {
    let mut buttons = HashSet::new();
    for (controller, control, pressed, _command) in BUTTON_BINDINGS {
      assert!(buttons.insert((controller, control, pressed)));
    }
  }

  #[test]
  fn character_bindings_are_unique() {
    let mut characters = HashSet::new();
    for (mode, c, modifiers, _command) in CHARACTER_BINDINGS {
      assert!(characters.insert((mode, c, modifiers)));
    }
  }

  #[test]
  fn character_bindings_are_uppercase() {
    for (_, c, _, _command) in CHARACTER_BINDINGS {
      let s = c.to_string();
      assert_eq!(s.to_uppercase(), s);
    }
  }

  #[test]
  fn encoder_bindings_are_unique() {
    let mut encoders = HashSet::new();
    for (controller, control, _command) in ENCODER_BINDINGS {
      assert!(encoders.insert((controller, control)));
    }
  }

  #[test]
  fn named_bindings_are_unique() {
    let mut names = HashSet::new();
    for (mode, name, modifiers, _command) in NAMED_BINDINGS {
      assert!(names.insert((mode, name, modifiers)));
    }
  }
}
