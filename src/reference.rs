use {super::*, std::sync::LazyLock};

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
      Vector2::new(resolution, resolution),
      None,
    ))
    .unwrap(),
  )
});

enum Error {
  Missing,
  Mismatch,
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
      state: State::default(),
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
        Error::Missing => panic!("no reference image found"),
        Error::Mismatch => panic!("reference image mismatch"),
      }
    }
  }

  #[track_caller]
  fn try_run(self) -> Result<(), Error> {
    let mut renderer = RENDERER.lock().unwrap();

    let resolution = self.resolution.unwrap_or(256);

    let width = self.width.unwrap_or(resolution).try_into().unwrap();

    let height = self.height.unwrap_or(resolution).try_into().unwrap();

    renderer.resize(Vector2::new(width, height), resolution.try_into().unwrap());

    renderer
      .render(&Analyzer::new(), &self.state, Instant::now())
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

  fn state(mut self, state: State) -> Self {
    self.state = state;
    self
  }

  fn width(mut self, width: u32) -> Self {
    self.width = Some(width);
    self
  }
}

#[test]
#[ignore]
fn circle() {
  let mut state = State::default();
  state.invert().circle().push();
  Test::new(name!()).state(state).run();
}

#[test]
#[ignore]
fn circle_small_even() {
  let mut state = State::default();
  state.invert().circle().push();
  Test::new(name!()).resolution(10).state(state).run();
}

#[test]
#[ignore]
fn circle_small_odd() {
  let mut state = State::default();
  state.invert().circle().push();
  Test::new(name!()).resolution(9).state(state).run();
}

#[test]
#[ignore]
fn circle_medium_even() {
  let mut state = State::default();
  state.invert().circle().push();
  Test::new(name!()).resolution(32).state(state).run();
}

#[test]
#[ignore]
fn circle_medium_odd() {
  let mut state = State::default();
  state.invert().circle().push();
  Test::new(name!()).resolution(31).state(state).run();
}

#[test]
#[ignore]
fn default() {
  Test::new(name!()).state(State::default()).run();
}

#[test]
#[ignore]
fn left() {
  let mut state = State::default();
  state.invert().left().push();
  Test::new(name!()).state(state).run();
}

#[test]
#[ignore]
fn x() {
  let mut state = State::default();
  state.invert().x().push();
  Test::new(name!()).state(state).run();
}

#[test]
#[ignore]
fn x_oblong() {
  let mut state = State::default();
  state.invert().x().push();
  Test::new(name!()).width(256).height(128).state(state).run();
}

#[test]
#[ignore]
fn x_small_even() {
  let mut state = State::default();
  state.invert().x().push();
  Test::new(name!()).resolution(10).state(state).run();
}

#[test]
#[ignore]
fn x_small_odd() {
  let mut state = State::default();
  state.invert().x().push();
  Test::new(name!()).resolution(9).state(state).run();
}

#[test]
#[ignore]
fn x_medium_even() {
  let mut state = State::default();
  state.invert().x().push();
  Test::new(name!()).resolution(32).state(state).run();
}

#[test]
#[ignore]
fn x_medium_odd() {
  let mut state = State::default();
  state.invert().x().push();
  Test::new(name!()).resolution(31).state(state).run();
}

#[test]
#[ignore]
fn tile() {
  let mut state = State::default();

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
fn circle_scale() {
  let mut state = State::default();
  state.invert().circle().scale(2.0).times(2);
  Test::new(name!()).state(state).run();
}

#[test]
#[ignore]
fn x_scale() {
  let mut state = State::default();
  state.repeat(false).invert().x().scale(2.0).times(2);
  Test::new(name!()).state(state).run();
}

#[test]
#[ignore]
fn x_scale_repeat() {
  let mut state = State::default();
  state.repeat(true).invert().x().scale(2.0).times(2);
  Test::new(name!()).state(state).run();
}

#[test]
#[ignore]
fn circle_scale_interpolated() {
  let mut state = State::default();
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
fn x_scale_interpolated() {
  let mut state = State::default();
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
fn sampling_repeat_on() {
  let mut state = State::default();
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
fn sampling_repeat_off() {
  let mut state = State::default();

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
fn cross() {
  let mut state = State::default();
  state.invert().cross().push();
  Test::new(name!()).state(state).run();
}

#[test]
#[ignore]
fn square() {
  let mut state = State::default();
  state.invert().square().push();
  Test::new(name!()).state(state).run();
}

#[test]
#[ignore]
fn triangle() {
  let mut state = State::default();
  state.invert().triangle().push();
  Test::new(name!()).state(state).run();
}

#[test]
#[ignore]
fn all() {
  let mut state = State::default();
  state.invert().all().push();
  Test::new(name!()).state(state).run();
}

#[test]
#[ignore]
fn none() {
  let mut state = State::default();
  state.invert().none().push();
  Test::new(name!()).state(state).run();
}

#[test]
#[ignore]
fn rotate_red() {
  let mut state = State::default();
  state.rotate_color(Axis::Red, TAU / 2.0).all().push();
  Test::new(name!()).state(state).run();
}

#[test]
#[ignore]
fn rotate_green() {
  let mut state = State::default();
  state.rotate_color(Axis::Green, TAU / 2.0).all().push();
  Test::new(name!()).state(state).run();
}

#[test]
#[ignore]
fn rotate_blue() {
  let mut state = State::default();
  state.rotate_color(Axis::Blue, TAU / 2.0).all().push();
  Test::new(name!()).state(state).run();
}

#[test]
#[ignore]
fn coordinates() {
  let mut state = State::default();
  state.coordinates(true).all().push();
  Test::new(name!()).state(state).run();
}

#[test]
#[ignore]
fn left_top() {
  let mut state = State::default();
  state.invert().left().push().top().push();
  Test::new(name!()).state(state).run();
}

#[test]
#[ignore]
fn mirror_x() {
  let mut state = State::default();
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
fn mirror_y() {
  let mut state = State::default();
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
fn mirror_x_inverse() {
  let mut state = State::default();
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
fn mirror_y_inverse() {
  let mut state = State::default();
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
fn mirror_xy() {
  let mut state = State::default();
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
  let mut state = State::default();
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
#[cfg(false)]
fn mirror_tile() {
  let mut state = State::default();

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
  let mut state = State::default();

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
fn rotation() {
  let mut state = State::default();

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
fn presets() {
  use tabled::{Table, Tabled, settings::style::Style};

  #[derive(Tabled)]
  struct Entry {
    name: &'static str,
    error: Error,
  }

  let mut errors = Vec::new();

  for preset in Preset::iter() {
    let mut state = State::default();
    state.filters.push(Preset::Test.filter());
    state.filters.push(preset.filter());

    if let Err(err) = Test::new(format!("preset-{preset}")).state(state).try_run() {
      errors.push(Entry {
        name: preset.name(),
        error: err,
      });
    }
  }

  if !errors.is_empty() {
    panic!("{}", Table::new(&errors).with(Style::sharp()));
  }
}
