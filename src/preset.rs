use super::*;

#[derive(Clone, Copy, Debug, EnumIter, ValueEnum, IntoStaticStr)]
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
  RotateBlaster,
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
      Self::RotateBlaster => Filter {
        color: color::rotate_hue_blaster(0.38 * TAU),
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

    const BLASTER: &[Preset] = &[
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
      Preset::RotateBlaster,
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

    if i == 0 {
      *BASE.choose(rng).unwrap()
    } else {
      *BLASTER.choose(rng).unwrap()
    }
  }
}

impl Display for Preset {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    f.write_str(self.name())
  }
}
