use {
  super::*,
  position::{bar, bbq, beat},
};

#[derive(Clone, Default)]
pub(crate) struct AllNight {
  extra: u32,
  index: u32,
}

impl Callback for AllNight {
  fn call(&mut self, state: &mut State, tick: Tick) {
    const EXTRA: &[Position] = &[
      bbq(25, 3, 3),
      bbq(29, 3, 3),
      bbq(31, 3, 3),
      bbq(33, 3, 3),
      bbq(37, 3, 3),
      bbq(46, 4, 3),
      bbq(48, 3, 3),
      bbq(65, 3, 3),
      bbq(69, 3, 3),
      bbq(73, 3, 3),
      bbq(77, 3, 3),
      bbq(81, 3, 3),
      bbq(85, 3, 3),
      bbq(86, 3, 3),
      bbq(87, 3, 3),
      bbq(89, 3, 3),
      bbq(93, 3, 3),
      bbq(94, 3, 3),
      bbq(95, 3, 3),
      bbq(105, 3, 3),
      bbq(109, 3, 3),
      bbq(110, 3, 3),
      bbq(111, 3, 3),
      bbq(113, 3, 3),
      bbq(117, 3, 3),
      bbq(118, 3, 3),
    ];

    let Some(position) = tick.advance() else {
      return;
    };

    if position >= bbq(126, 4, 1) {
      state.filters.clear();
      return;
    }

    let extra = EXTRA.contains(&position);

    if position.quarter() % 4 != 0 && !extra {
      return;
    }

    if position < bbq(17, 1, 1) && position.beat() % 4 != 0 {
      return;
    }

    let skip = [
      (bar(7) + beat(1), bar(9) + beat(1)),
      (bar(15) + beat(3), bar(17) + beat(1)),
      (bar(32) + beat(1), bar(33) + beat(1)),
      (bar(39) + beat(1), bar(41) + beat(1)),
      (bar(57) + beat(2), bar(65) + beat(1)),
      (bar(72) + beat(1), bar(73) + beat(1)),
      (bar(84) + beat(4), bar(85) + beat(2)),
      (bar(88) + beat(3), bar(89) + beat(1)),
      (bar(96) + beat(2), bar(105) + beat(1)),
      (bar(119) + beat(2), bar(121) + beat(1)),
      (bar(124) + beat(4), bar(125) + beat(1)),
      (bar(127) + beat(1), bar(140) + beat(1)),
    ];

    for (start, end) in skip {
      if (start.quarter()..end.quarter()).contains(&position.quarter()) {
        return;
      }
    }

    let c = (0xE000 + self.index % 376).try_into().unwrap();

    log::info!("codepoint: {}", Codepoint(c));

    state.filters.clear();

    if extra && self.extra < 4 {
      self.index -= 2;
      self.extra += 1;
    }

    state.filters.push(Filter {
      color: color::invert(),
      field: Field::texture(
        TextureField::default()
          .text(c)
          .font_stack(FontStack::Single(FontFamily::Named(Cow::Borrowed(
            "Last Resort Private",
          )))),
      ),
      position_response: Transformation2 {
        scaling: Vec2f::new(0.975, 0.975),
        ..default()
      },
      position: Mat3f::new_scaling(1.7777),
      ..default()
    });

    self.index += 1;
  }

  fn clone_box(&self) -> Box<dyn Callback> {
    Box::new(self.clone())
  }
}
