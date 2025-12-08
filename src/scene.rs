use super::*;

#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum Scene {
  All,
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

  pub(crate) fn state(self) -> State {
    match self {
      Self::All => {
        let mut state = State::default();
        state.invert().all().push();
        state
      }
      Self::Bottom => {
        let mut state = State::default();
        state.invert().bottom().push();
        state
      }
      Self::Circle => {
        let mut state = State::default();
        state.invert().circle().push();
        state
      }
      Self::Frequencies => {
        let mut state = State::default();
        state.invert().frequencies().push();
        state
      }
      Self::Hello => {
        let mut state = State::default();

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

        state
      }
      Self::Highwaystar => {
        let mut state = State::default();
        state
          .invert()
          .circle()
          .interpolate(true)
          .scale(2.0)
          .times(8);
        state
      }
      Self::Kaleidoscope => {
        let mut r = 0.0;
        let s = 1.0 / 0.75;

        let mut state = State::default();

        state
          .rotate_color(Axis::Green, 0.05 * TAU)
          .field(Field::Circle { size: Some(1.0) })
          .scale(s)
          .wrap(true)
          .repeat(true)
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

        state
      }
      Self::Middle => {
        let mut state = State::default();
        state.invert().top().push().bottom().push();
        state
      }
      Self::Noise => {
        let mut state = State::default();
        state
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
        state
      }
      Self::None => {
        let mut state = State::default();
        state.none();
        state
      }
      Self::Pattern => {
        let mut state = State::default();

        state
          .invert()
          .field(Field::Circle { size: Some(1.0) })
          .repeat(true)
          .alpha(0.75)
          .scale(2.0);

        for i in 0u8..8 {
          state.push().wrap(i.is_multiple_of(2));
        }

        state
      }
      Self::RedX => {
        let mut state = State::default();
        state.invert_r().x().push();
        state
      }
      Self::Rip => {
        let mut state = State::default();
        state.invert().top().push().samples().push();
        state
      }
      Self::Top => {
        let mut state = State::default();
        state.invert().top().push();
        state
      }
      Self::Samples => {
        let mut state = State::default();
        state.invert().samples().push();
        state
      }
      Self::Starburst => {
        let mut state = State::default();

        state
          .repeat(false)
          .wrap(false)
          .spread(true)
          .rotate_color(Axis::Green, 0.1 * TAU)
          .rotate_position(0.1 * TAU);

        for field in [
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
        ] {
          state.filter.field = field;
          state.push();
        }

        state
          .rotate_color(Axis::Blue, 0.1 * TAU)
          .rotate_position(0.2 * TAU);

        for field in [
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
        ] {
          state.filter.field = field;
          state.push();
        }

        state
      }
      Self::StarburstRandom => {
        let mut rng = SmallRng::from_rng(&mut rand::rng());

        let fields = [
          Field::All,
          Field::Circle { size: Some(1.0) },
          Field::Cross,
          Field::Square,
          Field::Top,
          Field::X,
        ];

        let mut state = State::default();

        state
          .repeat(false)
          .wrap(false)
          .spread(true)
          .rotate_color(Axis::Green, 0.1 * TAU)
          .rotate_position(0.1 * TAU);

        for _ in 0..20 {
          state.field(*fields.choose(&mut rng).unwrap()).push();
        }

        state
          .rotate_color(Axis::Blue, 0.1 * TAU)
          .rotate_position(0.2 * TAU);

        for _ in 0..10 {
          state.field(*fields.choose(&mut rng).unwrap()).push();
        }

        state
      }
      Self::X => {
        let mut state = State::default();
        state.invert().x().push();
        state
      }
    }
  }
}
