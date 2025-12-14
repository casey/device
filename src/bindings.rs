use super::*;

#[rustfmt::skip]
const BUTTON_BINDINGS: &[(Controller, u8, bool, Command)] = {
  use {Controller::*, generated::*};
  &[
    (Spectra,  0, true, PUSH_TOP),
    (Spectra,  1, true, PUSH_BOTTOM),
    (Spectra,  2, true, CYCLE),
    (Spectra,  3, true, CYCLE_ZOOM),

    (Spectra,  4, true, ROTATE_LEFT),
    (Spectra,  5, true, ROTATE_RIGHT),
    (Spectra,  6, true, SHUFFLE),
    (Spectra,  7, true, SWAP),

    (Spectra,  8, true, ADVANCE),
    (Spectra,  9, true, BLASTER),
    (Spectra, 10, true, WAFFLE),

    (Twister,  4, true, CLEAR_TRANSIENT_X_TRANSLATION),
    (Twister,  5, true, CLEAR_TRANSIENT_Y_TRANSLATION),
    (Twister,  6, true, CLEAR_TRANSIENT_SCALE),
  ]
};

#[rustfmt::skip]
const CHARACTER_BINDINGS: &[(ModeKind, char, ModifiersState, Command)] = {
  use {generated::*, ModeKind::*};

  const OFF: ModifiersState = ModifiersState::empty();
  const CTRL: ModifiersState = ModifiersState::CONTROL;
  const CTRL_SUPER: ModifiersState = CTRL.union(SUPER);
  const SHIFT: ModifiersState = ModifiersState::SHIFT;
  const SUPER: ModifiersState = ModifiersState::SUPER;

  &[
    (Normal, '+',  OFF,        INCREMENT_DB),
    (Normal, '-',  OFF,        DECREMENT_DB),
    (Normal, ':',  SHIFT,      ENTER_COMMAND_MODE),
    (Normal, '>',  SHIFT,      CAPTURE),
    (Normal, '?',  SHIFT,      PRINT),
    (Normal, 'A',  OFF,        ALL),
    (Normal, 'B',  OFF,        BLASTER),
    (Normal, 'C',  OFF,        CIRCLE),
    (Normal, 'D',  OFF,        COORDINATES),
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

  fn set_velocity_z(state: &mut State, value: u7) -> f32 {
    let value = float(value);
    state.velocity.z = value;
    value
  }

  &[
    (Twister, 0, set_alpha),
    (Twister, 1, set_db),
    (Twister, 4, set_velocity_x),
    (Twister, 5, set_velocity_y),
    (Twister, 6, set_velocity_z),
  ]
};

#[rustfmt::skip]
const NAMED_BINDINGS: &[(ModeKind, NamedKey, Command)] = {
  use {
    ModeKind::{Normal, Play, Command},
    NamedKey::*,
    generated::*,
  };

  &[
    (Command, Backspace,  POP_COMMAND),
    (Command, Enter,      EXECUTE_COMMAND),
    (Command, Escape,     ENTER_NORMAL_MODE),
    (Command, Tab,        COMPLETE_COMMAND),
    (Normal,  ArrowLeft,  NEGATIVE_ROTATION),
    (Normal,  ArrowRight, POSITIVE_ROTATION),
    (Normal,  Backspace,  POP),
    (Play,    Escape,     ENTER_NORMAL_MODE),
  ]
};

pub(crate) struct Bindings {
  button: HashMap<(Controller, u8, bool), Command>,
  character: HashMap<(ModeKind, String, ModifiersState), Command>,
  encoder: HashMap<(Controller, u8), fn(&mut State, u7) -> f32>,
  named: HashMap<(ModeKind, NamedKey), Command>,
}

impl Bindings {
  pub(crate) fn button(&self, controller: Controller, button: u8, press: bool) -> Option<Command> {
    let command = self.button.get(&(controller, button, press)).copied();

    if command.is_none() {
      log::info!("unbound button: {controller:?} {button} {press}");
    }

    command
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
      Key::Character(character) => self
        .character
        .get(&(mode, character.to_uppercase(), modifiers.state()))
        .copied(),
      Key::Named(named) => self.named.get(&(mode, *named)).copied(),
      _ => None,
    };

    if command.is_none() {
      log::info!("unbound key: {key:?} {modifiers:?}");
    }

    command
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
        .map(|(mode, named, command)| ((mode, named), command))
        .collect(),
    }
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
    for (mode, name, _command) in NAMED_BINDINGS {
      assert!(names.insert((mode, name)));
    }
  }
}
