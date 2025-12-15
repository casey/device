use super::*;

#[derive(Clone, Copy, Debug, EnumIter, ValueEnum, IntoStaticStr, PartialEq)]
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
  const BASE: &[Preset] = &[
    Preset::Circle,
    Preset::Cross,
    Preset::Left,
    Preset::Square,
    Preset::Test,
    Preset::Top,
    Preset::Triangle,
    Preset::X,
  ];

  pub(crate) const LIMIT: usize = 16;

  const REST: &[Self] = &[
    Preset::Circle,
    Preset::Cross,
    Preset::FlipHorizontal,
    Preset::FlipVertical,
    Preset::Invert,
    Preset::InvertBlue,
    Preset::InvertGreen,
    Preset::InvertRed,
    Preset::Jump,
    Preset::Left,
    Preset::MirrorHorizontal,
    Preset::MirrorVertical,
    Preset::Rotate,
    Preset::RotateBlaster,
    Preset::RotateBlue,
    Preset::RotateBlueVelocity,
    Preset::RotateGreen,
    Preset::RotateGreenVelocity,
    Preset::RotateRed,
    Preset::RotateRedResponsive,
    Preset::RotateRedVelocity,
    Preset::RotateResponsive,
    Preset::RotateVelocity,
    Preset::Scale,
    Preset::ScaleVelocity,
    Preset::Spin,
    Preset::Square,
    Preset::Test,
    Preset::Top,
    Preset::TranslateBlueVelocity,
    Preset::TranslateGreenVelocity,
    Preset::TranslateRedVelocity,
    Preset::TranslateVelocity,
    Preset::Triangle,
    Preset::X,
    Preset::ZoomInCenter,
    Preset::ZoomInCorner,
    Preset::ZoomOutCenter,
    Preset::ZoomOutCorner,
  ];

  pub(crate) fn filter(self) -> Filter {
    let mut filter = match self {
      Self::Circle => Filter {
        color: color::invert(),
        field: Field::Circle { size: None },
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
      *Self::BASE.choose(rng).unwrap()
    } else {
      *Self::REST.choose(rng).unwrap()
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
  fn base() {
    use Preset::*;

    let presets = Preset::iter()
      .filter(|preset| {
        !matches!(
          preset,
          Desaturate
            | FlipHorizontal
            | FlipVertical
            | Identity
            | Invert
            | InvertBlue
            | InvertGreen
            | InvertRed
            | Jump
            | MirrorHorizontal
            | MirrorVertical
            | Off
            | Rotate
            | RotateBlaster
            | RotateBlue
            | RotateBlueVelocity
            | RotateGreen
            | RotateGreenVelocity
            | RotateRed
            | RotateRedResponsive
            | RotateRedVelocity
            | RotateResponsive
            | RotateVelocity
            | Scale
            | ScaleVelocity
            | Spin
            | TranslateBlueVelocity
            | TranslateGreenVelocity
            | TranslateRedVelocity
            | TranslateVelocity
            | ZoomInCenter
            | ZoomInCorner
            | ZoomOutCenter
            | ZoomOutCorner
        )
      })
      .collect::<Vec<Preset>>();

    assert_eq!(Preset::BASE, presets);
  }

  #[test]
  fn rest() {
    use Preset::*;

    let presets = Preset::iter()
      .filter(|preset| !matches!(preset, Desaturate | Identity | Off))
      .collect::<Vec<Preset>>();

    assert_eq!(Preset::REST, presets);
  }
}
