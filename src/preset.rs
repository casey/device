use super::*;

#[derive(
  Clone, Copy, Debug, EnumIter, ValueEnum, IntoStaticStr, PartialEq, Ord, PartialOrd, Eq,
)]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum Preset {
  Circle,
  Cross,
  Desaturate,
  FlipHorizontal,
  FlipVertical,
  Identity,
  Invert,
  InvertBlue,
  InvertGreen,
  InvertRed,
  Jump,
  Left,
  MirrorHorizontal,
  MirrorVertical,
  Off,
  Rotate,
  RotateBlaster,
  RotateBlue,
  RotateBlueVelocity,
  RotateGreen,
  RotateGreenVelocity,
  RotateRed,
  RotateRedResponsive,
  RotateRedVelocity,
  RotateResponsive,
  RotateVelocity,
  Scale,
  ScaleVelocity,
  Spin,
  Square,
  Test,
  Top,
  TranslateBlueVelocity,
  TranslateGreenVelocity,
  TranslateRedVelocity,
  TranslateVelocity,
  Triangle,
  X,
  ZoomInCenter,
  ZoomInCorner,
  ZoomOutCenter,
  ZoomOutCorner,
}

impl Preset {
  const COLOR: &[Self] = &[
    Self::Invert,
    Self::InvertBlue,
    Self::InvertGreen,
    Self::InvertRed,
    Self::Rotate,
    Self::RotateBlaster,
    Self::RotateBlue,
    Self::RotateGreen,
    Self::RotateRed,
    Self::Test,
  ];

  const COLOR_RESPONSIVE: &[Self] = &[Self::RotateResponsive, Self::RotateRedResponsive];

  const COLOR_VELOCITY: &[Self] = &[
    Self::RotateBlueVelocity,
    Self::RotateGreenVelocity,
    Self::RotateRedVelocity,
    Self::TranslateBlueVelocity,
    Self::TranslateGreenVelocity,
    Self::TranslateRedVelocity,
  ];

  pub(crate) const LIMIT: usize = 16;

  const MOVEMENT_RESPONSIVE: &[Self] = &[Self::Spin, Self::Scale, Self::Jump];

  const MOVEMENT_VELOCITY: &[Self] = &[
    Self::RotateVelocity,
    Self::ScaleVelocity,
    Self::TranslateVelocity,
  ];

  const SHAPE: &[Self] = &[
    Self::Circle,
    Self::Cross,
    Self::Left,
    Self::Square,
    Self::Top,
    Self::Triangle,
    Self::X,
  ];

  const TRANSFORM: &[Self] = &[
    Self::FlipHorizontal,
    Self::FlipVertical,
    Self::MirrorHorizontal,
    Self::MirrorVertical,
    Self::ZoomInCenter,
    Self::ZoomInCorner,
    Self::ZoomOutCenter,
    Self::ZoomOutCorner,
    Self::Identity,
  ];

  pub(crate) fn filter(self) -> Filter {
    let mut filter = match self {
      Self::Circle => Filter {
        color: color::invert(),
        field: Field::Circle { radius: 0.5 },
        ..default()
      },
      Self::Cross => Filter {
        color: color::invert(),
        field: Field::Cross,
        ..default()
      },
      Self::Desaturate => Filter {
        color: color::saturate(0.0),
        ..default()
      },
      Self::FlipHorizontal => Filter {
        position: Mat3f::new_nonuniform_scaling(&Vec2f::new(-1.0, 1.0)),
        ..default()
      },
      Self::FlipVertical => Filter {
        position: Mat3f::new_nonuniform_scaling(&Vec2f::new(1.0, -1.0)),
        ..default()
      },
      Self::Identity => Filter::default(),
      Self::InvertBlue => Filter {
        color: Axis::Blue.invert(),
        ..default()
      },
      Self::Invert => Filter {
        color: color::invert(),
        ..default()
      },
      Self::InvertGreen => Filter {
        color: Axis::Green.invert(),
        ..default()
      },
      Self::InvertRed => Filter {
        color: Axis::Red.invert(),
        ..default()
      },
      Self::Left => Filter {
        color: color::invert(),
        field: Field::Left,
        ..default()
      },
      Self::MirrorHorizontal => Filter {
        mirror: Vector2::new(Mirror::Triangle, Mirror::Off),
        ..default()
      },
      Self::MirrorVertical => Filter {
        mirror: Vector2::new(Mirror::Off, Mirror::Inverse),
        ..default()
      },
      Self::Off => Filter {
        color: Mat4f::zeros(),
        ..default()
      },
      Self::Rotate => Filter {
        color: color::rotate_hue_yiq(0.38 * TAU),
        ..default()
      },
      Self::RotateBlue => Filter {
        color: Axis::Blue.rotate(0.38 * TAU),
        ..default()
      },
      Self::RotateBlaster => Filter {
        color: color::rotate_hue_blaster(0.38 * TAU),
        ..default()
      },
      Self::RotateGreen => Filter {
        color: Axis::Green.rotate(0.38 * TAU),
        ..default()
      },
      Self::RotateRed => Filter {
        color: Axis::Red.rotate(0.38 * TAU),
        ..default()
      },
      Self::RotateRedResponsive => Filter {
        color_response: Transformation3 {
          space: Space::CenteredRgb,
          rotation: UnitQuaternion::from_axis_angle(&Axis::Red.axis(), 0.38 * TAU),
          ..default()
        },
        ..default()
      },
      Self::RotateRedVelocity => Filter {
        color_velocity: Transformation3 {
          space: Space::CenteredRgb,
          rotation: UnitQuaternion::from_axis_angle(&Axis::Red.axis(), 0.1 * TAU),
          ..default()
        },
        ..default()
      },
      Self::RotateGreenVelocity => Filter {
        color_velocity: Transformation3 {
          space: Space::CenteredRgb,
          rotation: UnitQuaternion::from_axis_angle(&Axis::Green.axis(), 0.1 * TAU),
          ..default()
        },
        ..default()
      },
      Self::RotateBlueVelocity => Filter {
        color_velocity: Transformation3 {
          space: Space::CenteredRgb,
          rotation: UnitQuaternion::from_axis_angle(&Axis::Blue.axis(), 0.1 * TAU),
          ..default()
        },
        ..default()
      },
      Self::TranslateRedVelocity => Filter {
        color_velocity: Transformation3 {
          space: Space::CenteredRgb,
          translation: Vec3f::new(1.0 / TAU, 0.0, 0.0),
          sin: true,
          ..default()
        },
        ..default()
      },
      Self::TranslateGreenVelocity => Filter {
        color_velocity: Transformation3 {
          space: Space::CenteredRgb,
          translation: Vec3f::new(0.0, 1.0 / TAU, 0.0),
          sin: true,
          ..default()
        },
        ..default()
      },
      Self::TranslateBlueVelocity => Filter {
        color_velocity: Transformation3 {
          space: Space::CenteredRgb,
          translation: Vec3f::new(0.0, 0.0, 1.0 / TAU),
          sin: true,
          ..default()
        },
        ..default()
      },
      Self::RotateResponsive => Filter {
        color_response: Transformation3 {
          space: Space::Yiq,
          rotation: UnitQuaternion::from_axis_angle(&Vec3f::x_axis(), 0.38 * TAU),
          ..default()
        },
        ..default()
      },
      Self::Spin => Filter {
        position_response: Transformation2 {
          rotation: -0.5,
          ..default()
        },
        ..default()
      },
      Self::Scale => Filter {
        position_response: Transformation2 {
          scaling: Vec2f::new(2.0, 2.0),
          ..default()
        },
        ..default()
      },
      Self::Jump => Filter {
        position_response: Transformation2 {
          translation: Vec2f::new(1.0, 1.0),
          ..default()
        },
        ..default()
      },
      Self::Square => Filter {
        color: color::invert(),
        field: Field::Square,
        ..default()
      },
      Self::Test => Filter {
        grid: 10.0,
        grid_alpha: 1.0,
        ..default()
      },
      Self::Top => Filter {
        color: color::invert(),
        field: Field::Top,
        ..default()
      },
      Self::TranslateVelocity => Filter {
        position_velocity: Transformation2 {
          translation: Vec2f::new(-0.1, 0.0),
          ..default()
        },
        ..default()
      },
      Self::RotateVelocity => Filter {
        position_velocity: Transformation2 {
          rotation: 0.1,
          ..default()
        },
        ..default()
      },
      Self::ScaleVelocity => Filter {
        position_velocity: Transformation2 {
          scaling: Vec2f::new(1.1, 1.1),
          ..default()
        },
        ..default()
      },
      Self::Triangle => Filter {
        color: color::invert(),
        field: Field::Triangle,
        ..default()
      },
      Self::X => Filter {
        color: color::invert(),
        field: Field::X,
        ..default()
      },
      Self::ZoomInCenter => Filter {
        position: Mat3f::new_scaling(0.5),
        ..default()
      },
      Self::ZoomInCorner => Filter {
        position: Mat3f::new_scaling(0.5).append_translation(&Vec2f::new(0.5, 0.0)),
        ..default()
      },
      Self::ZoomOutCenter => Filter {
        position: Mat3f::new_scaling(2.0),
        ..default()
      },
      Self::ZoomOutCorner => Filter {
        position: Mat3f::new_scaling(2.0).prepend_translation(&Vec2f::new(0.5, 0.5)),
        ..default()
      },
    };

    filter.preset = Some(self);

    filter
  }

  pub(crate) fn name(self) -> &'static str {
    self.into()
  }

  pub(crate) fn random(rng: &mut SmallRng, i: usize) -> Self {
    if i == 0 {
      Self::SHAPE.choose(rng).copied().unwrap()
    } else {
      // Self::SHAPE
      //   .iter()
      //   .chain(Preset::COLOR)
      //   .chain(Preset::COLOR_RESPONSIVE)
      //   .chain(Preset::COLOR_VELOCITY)
      //   .chain(Preset::MOVEMENT_RESPONSIVE)
      //   .chain(Preset::MOVEMENT_VELOCITY)
      //   .chain(Preset::TRANSFORM)
      //   .copied()
      //   .collect::<Vec<Self>>()
      //   .choose(rng)
      //   .copied()
      //   .unwrap()
      Self::COLOR
        .iter()
        .chain(Preset::COLOR_RESPONSIVE)
        .chain(Preset::COLOR_VELOCITY)
        .chain(Preset::MOVEMENT_RESPONSIVE)
        .chain(Preset::MOVEMENT_VELOCITY)
        .chain(Preset::TRANSFORM)
        .copied()
        .collect::<Vec<Self>>()
        .choose(rng)
        .copied()
        .unwrap()
    }
  }

  pub(crate) fn random_black_and_white(rng: &mut SmallRng, i: usize) -> Self {
    if i == 0 {
      Self::SHAPE.choose(rng).copied().unwrap()
    } else {
      // *Self::SHAPE
      //   .iter()
      //   .chain(Self::MOVEMENT_RESPONSIVE)
      //   .chain(Self::MOVEMENT_VELOCITY)
      //   .chain(Self::TRANSFORM)
      //   .copied()
      //   .collect::<Vec<Self>>()
      //   .choose(rng)
      //   .unwrap()
      Self::MOVEMENT_RESPONSIVE
        .iter()
        .chain(Self::MOVEMENT_VELOCITY)
        .chain(Self::TRANSFORM)
        .copied()
        .collect::<Vec<Self>>()
        .choose(rng)
        .copied()
        .unwrap()
    }
  }
}

impl Display for Preset {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    f.write_str(self.name())
  }
}

impl From<Preset> for Filter {
  fn from(preset: Preset) -> Self {
    preset.filter()
  }
}

#[cfg(test)]
mod tests {
  use {super::*, pretty_assertions::assert_eq};

  #[test]
  fn categories() {
    const BORING: &[Preset] = &[Preset::Desaturate, Preset::Off];

    let mut categorized = BORING
      .iter()
      .chain(Preset::SHAPE)
      .chain(Preset::COLOR)
      .chain(Preset::COLOR_RESPONSIVE)
      .chain(Preset::COLOR_VELOCITY)
      .chain(Preset::TRANSFORM)
      .chain(Preset::MOVEMENT_RESPONSIVE)
      .chain(Preset::MOVEMENT_VELOCITY)
      .copied()
      .collect::<Vec<Preset>>();
    categorized.sort();
    assert_eq!(categorized, Preset::iter().collect::<Vec<Preset>>());
  }
}
