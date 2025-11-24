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
        .vz(-0.05)
        .interpolate(true)
        .position(Mat3f::new_rotation(-0.01))
        .none()
        .times(157),
      Self::None => State::default(),
      Self::RedX => State::default().invert_r().x().push(),
      Self::Rip => State::default().invert().top().push().samples().push(),
      Self::Top => State::default().invert().top().push(),
      Self::Samples => State::default().invert().samples().push(),
      Self::X => State::default().invert().x().push(),
    }
  }
}
