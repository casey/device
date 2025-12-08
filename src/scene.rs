use super::*;

#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum Scene {
  All,
  Bottom,
  Circle,
  Frequencies,
  Hello,
  Highwaystar,
  Middle,
  Noise,
  None,
  RedX,
  Rip,
  Samples,
  Starburst,
  StarburstRandom,
  Top,
  X,
}

impl Scene {
  pub(crate) fn state(self) -> State {
    match self {
      Self::All => State::default().invert().all().push(),
      Self::Bottom => State::default().invert().bottom().push(),
      Self::Circle => State::default().invert().circle().push(),
      Self::Frequencies => State::default().invert().frequencies().push(),
      Self::Hello => State::default()
        .text(Some(Text {
          size: 0.075,
          string: "hello world".into(),
          x: 0.10,
          y: -0.10,
        }))
        .db(-40.0)
        .invert()
        .frequencies()
        .push(),
      Self::Highwaystar => State::default()
        .invert()
        .circle()
        .interpolate(true)
        .scale(2.0)
        .times(8),
      Self::Middle => State::default().invert().top().push().bottom().push(),
      Self::Noise => State::default()
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
        .times(157),
      Self::None => State::default(),
      Self::RedX => State::default().invert_r().x().push(),
      Self::Rip => State::default().invert().top().push().samples().push(),
      Self::Top => State::default().invert().top().push(),
      Self::Samples => State::default().invert().samples().push(),
      Self::Starburst => {
        let mut state = State::default()
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
          state = state.push();
        }

        state = state
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
          state = state.push();
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

        let mut state = State::default()
          .repeat(false)
          .wrap(false)
          .spread(true)
          .rotate_color(Axis::Green, 0.1 * TAU)
          .rotate_position(0.1 * TAU);

        for _ in 0..20 {
          state.filter.field = *fields.choose(&mut rng).unwrap();
          state = state.push();
        }

        state = state
          .rotate_color(Axis::Blue, 0.1 * TAU)
          .rotate_position(0.2 * TAU);

        for _ in 0..10 {
          state.filter.field = *fields.choose(&mut rng).unwrap();
          state = state.push();
        }

        state
      }
      Self::X => State::default().invert().x().push(),
    }
  }
}
