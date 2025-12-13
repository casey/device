use super::*;

const BUTTON_BINDINGS: &[((Controller, u8, bool), fn(&mut State))] = {
  use {Controller::*, commands::*};
  &[
    ((Spectra, 0, true), top),
    ((Spectra, 1, true), bottom),
    ((Spectra, 2, true), x),
    ((Spectra, 3, true), circle),
    ((Spectra, 4, true), zoom_out),
    ((Spectra, 5, true), zoom_in),
    ((Spectra, 6, true), negative_x_translation),
    ((Spectra, 7, true), positive_x_translation),
    ((Spectra, 8, true), pop),
    ((Twister, 4, true), clear_transient_x_translation),
    ((Twister, 5, true), clear_transient_y_translation),
    ((Twister, 6, true), clear_transient_scale),
  ]
};

const CHARACTER_BINDINGS: &[(char, fn(&mut State))] = {
  use commands::*;
  &[
    ('+', increment_db),
    ('-', decrement_db),
    ('A', all),
    ('B', blaster),
    ('C', circle),
    ('D', coordinates),
    ('F', toggle_fit),
    ('I', toggle_interpolate),
    ('L', frequencies),
    ('N', none),
    ('R', toggle_repeat),
    ('S', samples),
    ('T', toggle_tile),
    ('W', toggle_wrap),
    ('X', x),
    ('Z', zoom_out),
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

const NAMED_BINDINGS: &[(NamedKey, fn(&mut State))] = {
  use {NamedKey::*, commands::*};
  &[
    (ArrowLeft, negative_rotation),
    (ArrowRight, positive_rotation),
    (Backspace, pop),
  ]
};

pub(crate) struct Bindings {
  button: HashMap<(Controller, u8, bool), fn(&mut State)>,
  character: HashMap<String, fn(&mut State)>,
  encoder: HashMap<(Controller, u8), fn(&mut State, Parameter)>,
  named: HashMap<NamedKey, fn(&mut State)>,
}

impl Bindings {
  pub(crate) fn button(
    &self,
    controller: Controller,
    button: u8,
    press: bool,
  ) -> Option<fn(&mut State)> {
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

  pub(crate) fn key(&self, key: &Key) -> Option<fn(&mut State)> {
    let command = match key {
      Key::Character(character) => self.character.get(&character.to_uppercase()).copied(),
      Key::Named(named) => self.named.get(named).copied(),
      _ => None,
    };

    if command.is_none() {
      log::info!("unbound key: {key:?}");
    }

    command
  }

  pub(crate) fn new() -> Self {
    Self {
      button: BUTTON_BINDINGS.iter().copied().collect(),
      character: CHARACTER_BINDINGS
        .iter()
        .map(|(character, command)| (character.to_string(), *command))
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
    for (c, _command) in CHARACTER_BINDINGS {
      let s = c.to_string();
      assert_eq!(s.to_uppercase(), s);
    }
  }

  #[test]
  fn character_bindings_are_unique() {
    let mut characters = HashSet::new();
    for (c, _command) in CHARACTER_BINDINGS {
      assert!(characters.insert(c));
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
