use fundsp::{
  audionode::AudioNode,
  audiounit::AudioUnit,
  combinator::An,
  prelude::{U0, U1, U2, split},
};

pub(crate) trait IntoStereo<Outputs> {
  fn into_stereo(self) -> Box<dyn AudioUnit>;
}

impl<T> IntoStereo<U1> for T
where
  T: AudioNode<Inputs = U0, Outputs = U1> + 'static,
{
  fn into_stereo(self) -> Box<dyn AudioUnit> {
    Box::new(An(self) >> split::<U2>())
  }
}

impl<T> IntoStereo<U2> for T
where
  T: AudioNode<Inputs = U0, Outputs = U2> + 'static,
{
  fn into_stereo(self) -> Box<dyn AudioUnit> {
    Box::new(An(self))
  }
}
