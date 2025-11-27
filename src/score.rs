use super::*;

#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum Score {
  BrownNoise,
  BusySignal,
  ClickTrack,
  PinkNoise,
  Silence,
  WhiteNoise,
}

struct Wrapper<T: fundsp::audionode::AudioNode>(fundsp::prelude::An<T>);

impl<T: fundsp::audionode::AudioNode> Source for Wrapper<T> {
  fn channels(&self) -> u16 {
    1
  }

  fn current_span_len(&self) -> Option<usize> {
    None
  }

  fn sample_rate(&self) -> u32 {
    44_100
  }

  fn total_duration(&self) -> Option<Duration> {
    None
  }
}

impl<T: fundsp::audionode::AudioNode> Iterator for Wrapper<T> {
  type Item = f32;

  fn next(&mut self) -> Option<f32> {
    Some(self.0.get_mono())
  }
}

// One click per second: a narrow pulse at 1 Hz, shaped as noise
fn click_1hz() -> fundsp::prelude::An<impl fundsp::prelude::AudioNode> {
  use fundsp::hacker32::*;

  let trigger = ramp_hz(1.0) >> shape_fn(|x: f32| if x < 0.010 { 1.0 } else { 0.0 });

  trigger * brown() * 0.5
}

impl Score {
  // todo: set sample rate
  pub(crate) fn source(self) -> Box<dyn Source + Send> {
    match self {
      Self::BusySignal => Box::new(Wrapper(
        (fundsp::hacker32::sine_hz(480.0) + fundsp::hacker32::sine_hz(620.0))
          * fundsp::hacker32::lfo(|t| if t % 1.0 < 0.5 { 1.0 } else { 0.0 })
          * 0.25,
      )),
      Self::BrownNoise => Box::new(Wrapper(fundsp::hacker32::brown() * 0.25)),
      Self::ClickTrack => Box::new(Wrapper(click_1hz())),
      Self::PinkNoise => Box::new(Wrapper(fundsp::hacker32::pink() * 0.25)),
      Self::Silence => Box::new(Wrapper(fundsp::hacker32::constant(0.0))),
      Self::WhiteNoise => Box::new(Wrapper(fundsp::hacker32::white() * 0.25)),
    }
  }

  pub(crate) fn foo(self) -> Box<dyn fundsp::audiounit::AudioUnit> {
    match self {
      Self::BusySignal => Box::new(
        (fundsp::hacker32::sine_hz(480.0) + fundsp::hacker32::sine_hz(620.0))
          * fundsp::hacker32::lfo(|t| if t % 1.0 < 0.5 { 1.0 } else { 0.0 })
          * 0.25,
      ),
      Self::BrownNoise => Box::new(fundsp::hacker32::brown() * 0.25),
      Self::ClickTrack => Box::new(click_1hz()),
      Self::PinkNoise => Box::new(fundsp::hacker32::pink() * 0.25),
      Self::Silence => Box::new(fundsp::hacker32::constant(0.0)),
      Self::WhiteNoise => Box::new(fundsp::hacker32::white() * 0.25),
    }
  }
}
