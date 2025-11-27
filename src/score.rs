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
      Self::ClickTrack => voice::BrownNoise::new()
        .envelope(0.001, 0.02, 0.000, 0.002)
        .cycle(2.0 / 3.0)
        .source(),
      Self::PinkNoise => Box::new(Wrapper(fundsp::hacker32::pink() * 0.25)),
      Self::Silence => Box::new(Wrapper(fundsp::hacker32::constant(0.0))),
      Self::WhiteNoise => Box::new(Wrapper(fundsp::hacker32::white() * 0.25)),
    }
  }
}
