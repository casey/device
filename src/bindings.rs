use super::*;

const BUTTON_BINDINGS: &[(Controller, u8, fn(&mut State))] = {
  use {Controller::*, commands::*};
  &[
    (Spectra, 0, top),
    (Spectra, 1, bottom),
    (Spectra, 2, x),
    (Spectra, 3, circle),
    (Spectra, 4, zoom_out),
    (Spectra, 5, zoom_in),
    (Spectra, 6, negative_x_translation),
    (Spectra, 7, positive_x_translation),
    (Spectra, 8, pop),
    (Twister, 4, clear_transient_x_translation),
    (Twister, 5, clear_transient_y_translation),
    (Twister, 6, clear_transient_scale),
  ]
};

const ENCODER_BINDINGS: &[(u8, fn(&mut State, Parameter))] = {
  use commands::*;
  &[]
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

const NAMED_BINDINGS: &[(NamedKey, fn(&mut State))] = {
  use {NamedKey::*, commands::*};
  &[
    (ArrowLeft, negative_rotation),
    (ArrowRight, positive_rotation),
    (Backspace, pop),
  ]
};

pub(crate) struct Bindings {
  character: HashMap<String, fn(&mut State)>,
  named: HashMap<NamedKey, fn(&mut State)>,
}

impl Bindings {
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

  pub(crate) fn button(&self, controller: Controller, button: u8) -> Option<fn(&mut State)> {
    todo!()
  }

  pub(crate) fn new() -> Self {
    let character = CHARACTER_BINDINGS
      .iter()
      .map(|(character, command)| (character.to_string(), *command))
      .collect();

    let named = NAMED_BINDINGS.iter().copied().collect();

    Self { character, named }
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
}
