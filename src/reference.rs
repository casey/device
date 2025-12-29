use {
  super::*,
  std::sync::LazyLock,
  tabled::{Table, Tabled, settings::style::Style},
};

macro_rules! name {
  () => {{
    fn f() {}
    std::any::type_name_of_val(&f)
      .rsplit("::")
      .skip(1)
      .next()
      .unwrap()
      .replace('_', "-")
  }};
}

static RENDERER: LazyLock<Mutex<Renderer>> = LazyLock::new(|| {
  let resolution = 256.try_into().unwrap();
  Mutex::new(
    pollster::block_on(Renderer::new(
      None,
      None,
      resolution,
      Size::new(resolution, resolution),
      None,
    ))
    .unwrap(),
  )
});

enum Error {
  Mismatch,
  Missing,
}

impl Display for Error {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Missing => f.write_str("missing"),
      Self::Mismatch => f.write_str("mismatch"),
    }
  }
}

struct Test {
  height: Option<u32>,
  name: String,
  resolution: Option<u32>,
  state: State,
  width: Option<u32>,
}

impl Test {
  fn height(mut self, height: u32) -> Self {
    self.height = Some(height);
    self
  }

  fn new(name: String) -> Self {
    Self {
      height: None,
      name,
      resolution: None,
      state: State::new(),
      width: None,
    }
  }

  fn resolution(mut self, resolution: u32) -> Self {
    self.resolution = Some(resolution);
    self
  }

  #[track_caller]
  fn run(self) {
    if let Err(err) = self.try_run() {
      match err {
        Error::Mismatch => panic!("reference image mismatch"),
        Error::Missing => panic!("no reference image found"),
      }
    }
  }

  fn state(mut self, state: State) -> Self {
    self.state = state;
    self
  }

  #[track_caller]
  fn try_run(self) -> Result<(), Error> {
    let mut renderer = RENDERER.lock().unwrap();

    let resolution = self.resolution.unwrap_or(256);

    let width = self.width.unwrap_or(resolution).try_into().unwrap();

    let height = self.height.unwrap_or(resolution).try_into().unwrap();

    renderer.resize(Size::new(width, height), resolution.try_into().unwrap());

    renderer
      .render(&Analyzer::new(), &self.state, None)
      .unwrap();

    let (tx, rx) = mpsc::channel();

    let expected = Utf8PathBuf::from(format!("reference/{}.png", self.name));
    let actual = expected.with_extension("test.png");

    renderer
      .capture(move |image| {
        image.save(&actual).unwrap();
        tx.send(image).unwrap();
      })
      .unwrap();

    renderer.poll().unwrap();

    drop(renderer);

    let actual = rx.recv().unwrap();

    if !expected.try_exists().unwrap() {
      return Err(Error::Missing);
    }

    if actual != Image::load(&expected).unwrap() {
      return Err(Error::Mismatch);
    }

    Ok(())
  }

  fn width(mut self, width: u32) -> Self {
    self.width = Some(width);
    self
  }
}

#[test]
#[ignore]
#[cfg(false)]
fn mirror_tile() {
  let mut state = State::new();
  state
    .invert()
    .left()
    .push()
    .top()
    .push()
    .mirror_x(Mirror::Triangle)
    .mirror_y(Mirror::Triangle)
    .circle()
    .push()
    .push()
    .push()
    .tile(true);
  Test::new(name!()).state(state).run();
}

#[test]
#[ignore]
#[cfg(false)]
fn zoom_tile() {
  let mut state = State::new();
  state
    .invert()
    .x()
    .push()
    .circle()
    .push()
    .square()
    .push()
    .position(Mat3f::new_scaling(2.0))
    .all()
    .push()
    .tile(true);
  Test::new(name!()).state(state).run();
}

#[test]
#[ignore]
fn all() {
  let mut state = State::new();
  state.invert().all().push();
  Test::new(name!()).state(state).run();
}

#[test]
#[ignore]
fn blend_mode_source() {
  let mut state = State::new();
  state.filter.field = Field::Texture;
  state.filter.media = Some(Media::new().text("😀").into());
  state.filter.blend_mode = BlendMode::Source;
  state.push();
  Test::new(name!()).state(state).run();
}

#[test]
#[ignore]
fn blend_mode_source_invert() {
  let mut state = State::new();
  state.filter.field = Field::Texture;
  state.filter.media = Some(Media::new().text("😀").into());
  state.filter.blend_mode = BlendMode::Source;
  state.invert().push();
  Test::new(name!()).state(state).run();
}

#[test]
#[ignore]
fn circle() {
  let mut state = State::new();
  state.invert().circle().push();
  Test::new(name!()).state(state).run();
}

#[test]
#[ignore]
fn circle_medium_even() {
  let mut state = State::new();
  state.invert().circle().push();
  Test::new(name!()).resolution(32).state(state).run();
}

#[test]
#[ignore]
fn circle_medium_odd() {
  let mut state = State::new();
  state.invert().circle().push();
  Test::new(name!()).resolution(31).state(state).run();
}

#[test]
#[ignore]
fn circle_scale() {
  let mut state = State::new();
  state.invert().circle().scale(2.0).times(2);
  Test::new(name!()).state(state).run();
}

#[test]
#[ignore]
fn circle_scale_interpolated() {
  let mut state = State::new();
  state
    .invert()
    .circle()
    .scale(2.0)
    .times(2)
    .interpolate(true);
  Test::new(name!()).state(state).run();
}

#[test]
#[ignore]
fn circle_small_even() {
  let mut state = State::new();
  state.invert().circle().push();
  Test::new(name!()).resolution(10).state(state).run();
}

#[test]
#[ignore]
fn circle_small_odd() {
  let mut state = State::new();
  state.invert().circle().push();
  Test::new(name!()).resolution(9).state(state).run();
}

#[test]
#[ignore]
fn coordinates() {
  let mut state = State::new();
  state.coordinates(true).all().push();
  Test::new(name!()).state(state).run();
}

#[test]
#[ignore]
fn cross() {
  let mut state = State::new();
  state.invert().cross().push();
  Test::new(name!()).state(state).run();
}

#[test]
#[ignore]
fn default_state() {
  Test::new(name!()).state(State::new()).run();
}

#[test]
#[ignore]
fn left() {
  let mut state = State::new();
  state.invert().left().push();
  Test::new(name!()).state(state).run();
}

#[test]
#[ignore]
fn left_top() {
  let mut state = State::new();
  state.invert().left().push().top().push();
  Test::new(name!()).state(state).run();
}

#[test]
#[ignore]
fn mirror_x() {
  let mut state = State::new();
  state
    .invert()
    .left()
    .push()
    .top()
    .push()
    .mirror_x(Mirror::Triangle)
    .identity()
    .all()
    .push();
  Test::new(name!()).state(state).run();
}

#[test]
#[ignore]
fn mirror_x_inverse() {
  let mut state = State::new();
  state
    .invert()
    .left()
    .push()
    .top()
    .push()
    .mirror_x(Mirror::Inverse)
    .identity()
    .all()
    .push();
  Test::new(name!()).state(state).run();
}

#[test]
#[ignore]
fn mirror_xy() {
  let mut state = State::new();
  state
    .invert()
    .left()
    .push()
    .top()
    .push()
    .mirror_x(Mirror::Triangle)
    .mirror_y(Mirror::Triangle)
    .identity()
    .all()
    .push();
  Test::new(name!()).state(state).run();
}

#[test]
#[ignore]
fn mirror_xy_field() {
  let mut state = State::new();
  state
    .invert()
    .left()
    .push()
    .top()
    .push()
    .mirror_x(Mirror::Triangle)
    .mirror_y(Mirror::Triangle)
    .circle()
    .push();
  Test::new(name!()).state(state).run();
}

#[test]
#[ignore]
fn mirror_y() {
  let mut state = State::new();
  state
    .invert()
    .left()
    .push()
    .top()
    .push()
    .mirror_y(Mirror::Triangle)
    .identity()
    .all()
    .push();
  Test::new(name!()).state(state).run();
}

#[test]
#[ignore]
fn mirror_y_inverse() {
  let mut state = State::new();
  state
    .invert()
    .left()
    .push()
    .top()
    .push()
    .mirror_y(Mirror::Inverse)
    .identity()
    .all()
    .push();
  Test::new(name!()).state(state).run();
}

#[test]
#[ignore]
fn none() {
  let mut state = State::new();
  state.invert().none().push();
  Test::new(name!()).state(state).run();
}

#[test]
#[ignore]
fn presets() {
  #[derive(Tabled)]
  #[allow(clippy::arbitrary_source_item_ordering)]
  struct Entry {
    name: &'static str,
    error: Error,
  }
  let mut errors = Vec::new();
  for preset in Preset::iter() {
    let mut state = State::new();
    let mut rng = SmallRng::from_seed(default());
    state.filters.push(Preset::Grid.filter(&mut rng));
    state.filters.push(preset.filter(&mut rng));
    if let Err(err) = Test::new(format!("preset-{preset}")).state(state).try_run() {
      errors.push(Entry {
        name: preset.name(),
        error: err,
      });
    }
  }
  assert!(
    errors.is_empty(),
    "{}",
    Table::new(&errors).with(Style::sharp()),
  );
}

#[test]
#[ignore]
fn rotate_blue() {
  let mut state = State::new();
  state.rotate_color(Axis::Blue, TAU / 2.0).all().push();
  Test::new(name!()).state(state).run();
}

#[test]
#[ignore]
fn rotate_green() {
  let mut state = State::new();
  state.rotate_color(Axis::Green, TAU / 2.0).all().push();
  Test::new(name!()).state(state).run();
}

#[test]
#[ignore]
fn rotate_red() {
  let mut state = State::new();
  state.rotate_color(Axis::Red, TAU / 2.0).all().push();
  Test::new(name!()).state(state).run();
}

#[test]
#[ignore]
fn rotation() {
  let mut state = State::new();
  state
    .rotation(TAU)
    .rms(Mat1x2f::new(0.0, 0.1))
    .invert()
    .x()
    .push();
  Test::new(name!()).state(state).run();
}

#[test]
#[ignore]
fn sampling_repeat_off() {
  let mut state = State::new();
  state
    .repeat(false)
    .rotate_position(0.2 * TAU)
    .rotate_color(Axis::Green, 0.1 * TAU)
    .all()
    .push()
    .push();
  Test::new(name!()).state(state).run();
}

#[test]
#[ignore]
fn sampling_repeat_on() {
  let mut state = State::new();
  state
    .repeat(true)
    .rotate_position(0.2 * TAU)
    .rotate_color(Axis::Green, 0.1 * TAU)
    .all()
    .push()
    .push();
  Test::new(name!()).state(state).run();
}

#[test]
#[ignore]
fn scenes() {
  #[derive(Tabled)]
  #[allow(clippy::arbitrary_source_item_ordering)]
  struct Entry {
    name: &'static str,
    error: Error,
  }
  let mut errors = Vec::new();
  for scene in Scene::iter() {
    let state = scene.state(&mut SmallRng::seed_from_u64(0));
    if let Err(err) = Test::new(format!("scene-{scene}")).state(state).try_run() {
      errors.push(Entry {
        name: scene.name(),
        error: err,
      });
    }
  }
  assert!(
    errors.is_empty(),
    "{}",
    Table::new(&errors).with(Style::sharp()),
  );
}

#[test]
#[ignore]
fn square() {
  let mut state = State::new();
  state.invert().square().push();
  Test::new(name!()).state(state).run();
}

#[test]
#[ignore]
fn status() {
  Test::new(name!())
    .state(State {
      status: true,
      ..default()
    })
    .run();
}

#[test]
#[ignore]
fn status_capture() {
  Test::new(name!())
    .state(State {
      capture_status: true,
      status: true,
      ..default()
    })
    .run();
}

#[test]
#[ignore]
fn status_capture_fill() {
  Test::new(name!())
    .width(256)
    .height(128)
    .state(State {
      capture_status: true,
      status: true,
      viewport: Viewport::Fill {
        position: Vec2f::new(-1.0, -1.0),
      },
      ..default()
    })
    .run();
}

#[test]
#[ignore]
fn texture() {
  let mut state = State::new();
  state.filter.field = Field::Texture;
  state.filter.media = Some(Media::new().text("A").into());
  state.invert().push();
  Test::new(name!()).state(state).run();
}

#[test]
#[ignore]
fn texture_bottom() {
  let mut state = State::new();
  state.filter.field = Field::Texture;
  state.filter.media = Some(
    Media::new()
      .text("A")
      .position(Vec2f::new(0.0, 0.5))
      .scale(0.5)
      .into(),
  );
  state.invert().push();
  Test::new(name!()).state(state).run();
}

#[test]
#[ignore]
fn texture_left() {
  let mut state = State::new();
  state.filter.field = Field::Texture;
  state.filter.media = Some(
    Media::new()
      .position(Vec2f::new(-0.5, 0.0))
      .scale(0.5)
      .text("A")
      .into(),
  );
  state.invert().push();
  Test::new(name!()).state(state).run();
}

#[test]
#[ignore]
fn texture_right() {
  let mut state = State::new();
  state.filter.field = Field::Texture;
  state.filter.media = Some(
    Media::new()
      .position(Vec2f::new(0.5, 0.0))
      .scale(0.5)
      .text("A")
      .into(),
  );
  state.invert().push();
  Test::new(name!()).state(state).run();
}

#[test]
#[ignore]
fn texture_small() {
  let mut state = State::new();
  state.filter.field = Field::Texture;
  state.filter.media = Some(Media::new().scale(0.5).text("A").into());
  state.invert().push();
  Test::new(name!()).state(state).run();
}

#[test]
#[ignore]
fn texture_top() {
  let mut state = State::new();
  state.filter.field = Field::Texture;
  state.filter.media = Some(
    Media::new()
      .position(Vec2f::new(0.0, -0.5))
      .scale(0.5)
      .text("A")
      .into(),
  );
  state.invert().push();
  Test::new(name!()).state(state).run();
}

#[test]
#[ignore]
fn tile() {
  let mut state = State::new();
  state
    .invert()
    .x()
    .push()
    .circle()
    .push()
    .x()
    .push()
    .square()
    .push()
    .circle()
    .push()
    .triangle()
    .push()
    .square()
    .push()
    .tile(true);
  Test::new(name!()).state(state).run();
}

#[test]
#[ignore]
fn triangle() {
  let mut state = State::new();
  state.invert().triangle().push();
  Test::new(name!()).state(state).run();
}

#[test]
#[ignore]
fn x() {
  let mut state = State::new();
  state.invert().x().push();
  Test::new(name!()).state(state).run();
}

#[test]
#[ignore]
fn x_medium_even() {
  let mut state = State::new();
  state.invert().x().push();
  Test::new(name!()).resolution(32).state(state).run();
}

#[test]
#[ignore]
fn x_medium_odd() {
  let mut state = State::new();
  state.invert().x().push();
  Test::new(name!()).resolution(31).state(state).run();
}

#[test]
#[ignore]
fn x_scale() {
  let mut state = State::new();
  state.repeat(false).invert().x().scale(2.0).times(2);
  Test::new(name!()).state(state).run();
}

#[test]
#[ignore]
fn x_scale_interpolated() {
  let mut state = State::new();
  state
    .repeat(false)
    .invert()
    .x()
    .scale(2.0)
    .times(2)
    .interpolate(true);
  Test::new(name!()).state(state).run();
}
#[test]
#[ignore]
fn x_scale_repeat() {
  let mut state = State::new();
  state.repeat(true).invert().x().scale(2.0).times(2);
  Test::new(name!()).state(state).run();
}

#[test]
#[ignore]
fn x_small_even() {
  let mut state = State::new();
  state.invert().x().push();
  Test::new(name!()).resolution(10).state(state).run();
}

#[test]
#[ignore]
fn x_small_odd() {
  let mut state = State::new();
  state.invert().x().push();
  Test::new(name!()).resolution(9).state(state).run();
}

#[test]
#[ignore]
fn x_tall() {
  let mut state = State::new();
  state.invert().x().push();
  Test::new(name!()).width(128).height(256).state(state).run();
}

#[test]
#[ignore]
fn x_tall_fit() {
  let mut state = State::new();
  state.invert().x().push();
  state.viewport = Viewport::Fit;
  Test::new(name!()).width(128).height(256).state(state).run();
}

#[test]
#[ignore]
fn x_wide() {
  let mut state = State::new();
  state.invert().x().push();
  Test::new(name!()).width(256).height(128).state(state).run();
}

#[test]
#[ignore]
fn x_wide_fit() {
  let mut state = State::new();
  state.invert().x().push();
  state.viewport = Viewport::Fit;
  Test::new(name!()).width(256).height(128).state(state).run();
}

#[test]
#[ignore]
fn zero_base() {
  let mut state = State::new();
  state.filters.push(Filter {
    field: Field::X,
    color: color::invert(),
    ..default()
  });
  state.filters.push(Filter {
    field: Field::Left,
    color: color::invert(),
    base: 0.0,
    ..default()
  });
  Test::new(name!()).state(state).run();
}
