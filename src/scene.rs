use super::*;

#[derive(Clone, Copy, EnumIter, ValueEnum, IntoStaticStr)]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum Scene {
  All,
  Blaster,
  Bottom,
  Circle,
  Frequencies,
  Hello,
  Highwaystar,
  Kaleidoscope,
  Middle,
  Noise,
  None,
  Pattern,
  RedX,
  Rip,
  Samples,
  Starburst,
  StarburstRandom,
  Top,
  X,
}

impl Scene {
  pub(crate) fn format(self) -> Option<Format> {
    match self {
      Self::Kaleidoscope | Self::Starburst | Self::StarburstRandom | Self::Pattern => {
        Some(Format::Bgra8Unorm)
      }
      _ => None,
    }
  }

  pub(crate) fn name(self) -> &'static str {
    self.into()
  }

  pub(crate) fn state(self, seed: Option<u64>) -> State {
    let mut state = State::default();

    let mut rng = if let Some(seed) = seed {
      SmallRng::seed_from_u64(seed)
    } else {
      SmallRng::from_rng(&mut rand::rng())
    };

    match self {
      Self::All => {
        state.invert().all().push();
      }
      Self::Blaster => {
        state.interpolate = rng.random();

        state
          .filters
          .extend((0..16).map(|i| Preset::random(&mut rng, i).filter()));
      }
      Self::Bottom => {
        state.invert().bottom().push();
      }
      Self::Circle => {
        state.invert().circle().push();
      }
      Self::Frequencies => {
        state.invert().frequencies().push();
      }
      Self::Hello => {
        state
          .text(Some(Text {
            size: 0.075,
            string: "hello world".into(),
            x: 0.10,
            y: -0.10,
          }))
          .db(-40.0)
          .invert()
          .frequencies()
          .push();
      }
      Self::Highwaystar => {
        state
          .repeat(false)
          .invert()
          .circle()
          .interpolate(true)
          .scale(2.0)
          .times(8);
      }
      Self::Kaleidoscope => {
        let mut r = 0.0;
        let s = 1.0 / 0.75;

        state
          .rotate_color(Axis::Green, 0.05 * TAU)
          .field(Field::Circle { size: Some(1.0) })
          .scale(s)
          .wrap(true)
          .db(-24.0)
          .times(8);

        state.callback(move |state, elapsed| {
          r += elapsed / 32.6 * TAU / 4.0;

          state
            .truncate(8)
            .transform(r, s)
            .rotate_color(Axis::Blue, 0.05 * TAU)
            .times(8);
        });
      }
      Self::Middle => {
        state.invert().top().push().bottom().push();
      }
      Self::Noise => {
        state
          .repeat(false)
          .invert()
          .x()
          .push()
          .samples()
          .push()
          .z(0.05)
          .vz(-0.05)
          .interpolate(true)
          .position(Mat3f::new_rotation(-0.01))
          .all()
          .identity()
          .times(157);
      }
      Self::None => {
        state.none();
      }
      Self::Pattern => {
        state
          .invert()
          .field(Field::Circle { size: Some(1.0) })
          .alpha(0.75)
          .scale(2.0);

        for i in 0u8..8 {
          state.push().wrap(i.is_multiple_of(2));
        }
      }
      Self::RedX => {
        state.invert_r().x().push();
      }
      Self::Rip => {
        state.invert().top().push().bottom().push().samples().push();
      }
      Self::Top => {
        state.invert().top().push();
      }
      Self::Samples => {
        state.invert().samples().push();
      }
      Self::Starburst => {
        const A: [Field; 20] = [
          Field::Cross,
          Field::Cross,
          Field::X,
          Field::Top,
          Field::All,
          Field::Circle { size: Some(1.0) },
          Field::All,
          Field::Cross,
          Field::Square,
          Field::All,
          Field::Cross,
          Field::Cross,
          Field::All,
          Field::Square,
          Field::Top,
          Field::Circle { size: Some(1.0) },
          Field::Top,
          Field::All,
          Field::X,
          Field::Cross,
        ];

        const B: [Field; 10] = [
          Field::Cross,
          Field::Circle { size: Some(1.0) },
          Field::Top,
          Field::Circle { size: Some(1.0) },
          Field::Top,
          Field::Circle { size: Some(1.0) },
          Field::X,
          Field::X,
          Field::Cross,
          Field::X,
        ];

        state
          .repeat(false)
          .wrap(false)
          .spread(true)
          .rotate_color(Axis::Green, 0.1 * TAU)
          .rotate_position(0.1 * TAU);

        for field in A {
          state.filter.field = field;
          state.push();
        }

        state
          .rotate_color(Axis::Blue, 0.1 * TAU)
          .rotate_position(0.2 * TAU);

        for field in B {
          state.filter.field = field;
          state.push();
        }
      }
      Self::StarburstRandom => {
        const FIELDS: [Field; 6] = [
          Field::All,
          Field::Circle { size: Some(1.0) },
          Field::Cross,
          Field::Square,
          Field::Top,
          Field::X,
        ];

        state
          .repeat(false)
          .wrap(false)
          .spread(true)
          .rotate_color(Axis::Green, 0.1 * TAU)
          .rotate_position(0.1 * TAU);

        for _ in 0..20 {
          state.field(*FIELDS.choose(&mut rng).unwrap()).push();
        }

        state
          .rotate_color(Axis::Blue, 0.1 * TAU)
          .rotate_position(0.2 * TAU);

        for _ in 0..10 {
          state.field(*FIELDS.choose(&mut rng).unwrap()).push();
        }
      }
      Self::X => {
        state.invert().x().push();
      }
    }

    state
  }
}

impl Display for Scene {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    f.write_str(self.name())
  }
}
