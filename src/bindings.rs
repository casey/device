use super::*;

const BUTTON_BINDINGS: &[((Controller, u8, bool), Command)] = {
  use {Controller::*, generated::*};
  &[
    ((Spectra, 0, true), TOP),
    ((Spectra, 1, true), BOTTOM),
    ((Spectra, 2, true), X),
    ((Spectra, 3, true), CIRCLE),
    ((Spectra, 4, true), ZOOM_OUT),
    ((Spectra, 5, true), ZOOM_IN),
    ((Spectra, 6, true), NEGATIVE_X_TRANSLATION),
    ((Spectra, 7, true), POSITIVE_X_TRANSLATION),
    ((Spectra, 8, true), POP),
    ((Twister, 4, true), CLEAR_TRANSIENT_X_TRANSLATION),
    ((Twister, 5, true), CLEAR_TRANSIENT_Y_TRANSLATION),
    ((Twister, 6, true), CLEAR_TRANSIENT_SCALE),
  ]
};

#[rustfmt::skip]
const CHARACTER_BINDINGS: &[(char, ModifiersState, Command)] = {
  use generated::*;

  const OFF: ModifiersState = ModifiersState::empty();
  const CTRL: ModifiersState = ModifiersState::CONTROL;
  const CTRL_SUPER: ModifiersState = CTRL.union(SUPER);
  const SHIFT: ModifiersState = ModifiersState::SHIFT;
  const SUPER: ModifiersState = ModifiersState::SUPER;

  &[
    ('+', OFF,        INCREMENT_DB),
    ('-', OFF,        DECREMENT_DB),
    (':', SHIFT,      ENTER_COMMAND_MODE),
    ('>', SHIFT,      CAPTURE),
    ('A', OFF,        ALL),
    ('B', OFF,        BLASTER),
    ('C', OFF,        CIRCLE),
    ('D', OFF,        COORDINATES),
    ('F', CTRL_SUPER, TOGGLE_FULLSCREEN),
    ('F', OFF,        TOGGLE_FIT),
    ('I', OFF,        TOGGLE_INTERPOLATE),
    ('L', OFF,        FREQUENCIES),
    ('N', OFF,        NONE),
    ('P', OFF,        ENTER_PLAY_MODE),
    ('R', OFF,        TOGGLE_REPEAT),
    ('R', SHIFT,      RELOAD_SHADERS),
    ('S', OFF,        SAMPLES),
    ('T', OFF,        TOGGLE_TILE),
    ('W', OFF,        TOGGLE_WRAP),
    ('X', OFF,        X),
    ('Z', OFF,        ZOOM_OUT),
  ]
};

const ENCODER_BINDINGS: &[((Controller, u8), fn(&mut State, Parameter))] = {
  use Controller::*;

  fn set_alpha(state: &mut State, parameter: Parameter) {
    state.alpha = parameter;
  }

  fn set_db(state: &mut State, parameter: Parameter) {
    state.db = parameter.value() as f32;
  }

  fn set_velocity_x(state: &mut State, parameter: Parameter) {
    state.velocity.x = parameter.bipolar();
  }

  fn set_velocity_y(state: &mut State, parameter: Parameter) {
    state.velocity.y = parameter.bipolar();
  }

  fn set_velocity_z(state: &mut State, parameter: Parameter) {
    state.velocity.z = parameter.bipolar();
  }

  &[
    ((Twister, 0), set_alpha),
    ((Twister, 1), set_db),
    ((Twister, 4), set_velocity_x),
    ((Twister, 5), set_velocity_y),
    ((Twister, 6), set_velocity_z),
  ]
};

const NAMED_BINDINGS: &[(NamedKey, Command)] = {
  use {NamedKey::*, generated::*};
  &[
    (ArrowLeft, NEGATIVE_ROTATION),
    (ArrowRight, POSITIVE_ROTATION),
    (Backspace, POP),
  ]
};

pub(crate) struct Bindings {
  button: HashMap<(Controller, u8, bool), Command>,
  character: HashMap<(String, ModifiersState), Command>,
  encoder: HashMap<(Controller, u8), fn(&mut State, Parameter)>,
  named: HashMap<NamedKey, Command>,
}

impl Bindings {
  pub(crate) fn button(&self, controller: Controller, button: u8, press: bool) -> Option<Command> {
    let command = self.button.get(&(controller, button, press)).copied();

    if command.is_none() {
      log::info!("unbound button: {controller:?} {button}");
    }

    command
  }

  pub(crate) fn encoder(
    &self,
    controller: Controller,
    encoder: u8,
  ) -> Option<fn(&mut State, Parameter)> {
    let command = self.encoder.get(&(controller, encoder)).copied();

    if command.is_none() {
      log::info!("unbound encoder: {controller:?} {encoder}");
    }

    command
  }

  pub(crate) fn key(&self, key: &Key, modifiers: Modifiers) -> Option<Command> {
    let command = match key {
      Key::Character(character) => self
        .character
        .get(&(character.to_uppercase(), modifiers.state()))
        .copied(),
      Key::Named(named) => self.named.get(named).copied(),
      _ => None,
    };

    if command.is_none() {
      log::info!("unbound key: {key:?} {modifiers:?}");
    }

    command
  }

  pub(crate) fn new() -> Self {
    Self {
      button: BUTTON_BINDINGS.iter().copied().collect(),
      character: CHARACTER_BINDINGS
        .iter()
        .map(|(character, modifiers, command)| ((character.to_string(), *modifiers), *command))
        .collect(),
      encoder: ENCODER_BINDINGS.iter().copied().collect(),
      named: NAMED_BINDINGS.iter().copied().collect(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn character_bindings_are_uppercase() {
    for (c, _, _command) in CHARACTER_BINDINGS {
      let s = c.to_string();
      assert_eq!(s.to_uppercase(), s);
    }
  }

  #[test]
  fn character_bindings_are_unique() {
    let mut characters = HashSet::new();
    for (c, modifiers, _command) in CHARACTER_BINDINGS {
      assert!(characters.insert((c, modifiers)));
    }
  }

  #[test]
  fn named_bindings_are_unique() {
    let mut names = HashSet::new();
    for (name, _command) in NAMED_BINDINGS {
      assert!(names.insert(name));
    }
  }

  #[test]
  fn button_bindings_are_unique() {
    let mut buttons = HashSet::new();
    for (button, _command) in BUTTON_BINDINGS {
      assert!(buttons.insert(button));
    }
  }

  #[test]
  fn encoder_bindings_are_unique() {
    let mut encoders = HashSet::new();
    for (encoder, _command) in ENCODER_BINDINGS {
      assert!(encoders.insert(encoder));
    }
  }
}
