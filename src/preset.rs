use super::*;

#[derive(Clone, Copy, Debug, EnumIter, ValueEnum, IntoStaticStr, PartialEq)]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum Preset {
  Circle,
  Cross,
  Desaturate,
  FlipH,
  FlipV,
  Identity,
  InvertB,
  InvertF,
  InvertG,
  InvertR,
  Left,
  MirrorH,
  MirrorV,
  Off,
  Rotate,
  RotateB,
  RotateBlaster,
  RotateG,
  RotateR,
  RotateRedResponsive,
  RotateResponsive,
  Spin,
  Square,
  Test,
  Top,
  Triangle,
  X,
  ZoomIn,
  ZoomInNe,
  ZoomOut,
  ZoomOutNe,
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
    Preset::FlipH,
    Preset::FlipV,
    Preset::InvertB,
    Preset::InvertF,
    Preset::InvertG,
    Preset::InvertR,
    Preset::Left,
    Preset::MirrorH,
    Preset::MirrorV,
    Preset::Rotate,
    Preset::RotateB,
    Preset::RotateBlaster,
    Preset::RotateG,
    Preset::RotateR,
    Preset::RotateRedResponsive,
    Preset::RotateResponsive,
    Preset::Spin,
    Preset::Square,
    Preset::Test,
    Preset::Top,
    Preset::Triangle,
    Preset::X,
    Preset::ZoomIn,
    Preset::ZoomInNe,
    Preset::ZoomOut,
    Preset::ZoomOutNe,
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
      Self::FlipH => Filter {
        position: Mat3f::new_nonuniform_scaling(&Vec2f::new(-1.0, 1.0)),
        ..default()
      },
      Self::FlipV => Filter {
        position: Mat3f::new_nonuniform_scaling(&Vec2f::new(1.0, -1.0)),
        ..default()
      },
      Self::Identity => Filter::default(),
      Self::InvertB => Filter {
        color: Axis::Blue.invert(),
        ..default()
      },
      Self::InvertF => Filter {
        color: color::invert(),
        ..default()
      },
      Self::InvertG => Filter {
        color: Axis::Green.invert(),
        ..default()
      },
      Self::InvertR => Filter {
        color: Axis::Red.invert(),
        ..default()
      },
      Self::Left => Filter {
        color: color::invert(),
        field: Field::Left,
        ..default()
      },
      Self::MirrorH => Filter {
        mirror: Vector2::new(Mirror::Triangle, Mirror::Off),
        ..default()
      },
      Self::MirrorV => Filter {
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
      Self::RotateB => Filter {
        color: Axis::Blue.rotate(0.38 * TAU),
        ..default()
      },
      Self::RotateBlaster => Filter {
        color: color::rotate_hue_blaster(0.38 * TAU),
        ..default()
      },
      Self::RotateG => Filter {
        color: Axis::Green.rotate(0.38 * TAU),
        ..default()
      },
      Self::RotateR => Filter {
        color: Axis::Red.rotate(0.38 * TAU),
        ..default()
      },
      Self::RotateRedResponsive => Filter {
        color_response: Transformation {
          space: Space::CenteredRgb,
          rotation: UnitQuaternion::from_axis_angle(&Axis::Red.axis(), 0.38 * TAU),
          ..default()
        },
        ..default()
      },
      Self::RotateResponsive => Filter {
        color_response: Transformation {
          space: Space::Yiq,
          rotation: UnitQuaternion::from_axis_angle(&Vec3f::x_axis(), 0.38 * TAU),
          ..default()
        },
        ..default()
      },
      Self::Spin => Filter {
        rotation: -0.5,
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
      Self::ZoomIn => Filter {
        position: Mat3f::new_scaling(0.5),
        ..default()
      },
      Self::ZoomInNe => Filter {
        position: Mat3f::new_scaling(0.5).append_translation(&Vec2f::new(0.5, 0.0)),
        ..default()
      },
      Self::ZoomOut => Filter {
        position: Mat3f::new_scaling(2.0),
        ..default()
      },
      Self::ZoomOutNe => Filter {
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
            | FlipH
            | FlipV
            | Identity
            | InvertB
            | InvertF
            | InvertG
            | InvertR
            | MirrorH
            | MirrorV
            | Off
            | Rotate
            | RotateB
            | RotateBlaster
            | RotateG
            | RotateR
            | RotateResponsive
            | RotateRedResponsive
            | Spin
            | ZoomIn
            | ZoomInNe
            | ZoomOut
            | ZoomOutNe
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
