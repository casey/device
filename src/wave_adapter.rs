use super::*;

pub(crate) struct WaveAdapter(pub(crate) Wave);

impl WaveAdapter {
  pub(crate) fn into_inner(self) -> Wave {
    self.0
  }
}

impl Adapter<'static, f32> for WaveAdapter {
  fn channels(&self) -> usize {
    self.0.channels()
  }

  fn frames(&self) -> usize {
    self.0.len()
  }

  unsafe fn read_sample_unchecked(&self, channel: usize, frame: usize) -> f32 {
    self.0.at(channel, frame)
  }
}

impl AdapterMut<'static, f32> for WaveAdapter {
  unsafe fn write_sample_unchecked(&mut self, channel: usize, frame: usize, value: &f32) -> bool {
    self.0.set(channel, frame, *value);
    false
  }
}
