use super::*;

pub(crate) struct StreamConfigDisplay<'a>(pub(crate) &'a StreamConfig);

impl Display for StreamConfigDisplay<'_> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(
      f,
      "{}x{}x{}",
      self.0.sample_rate.0,
      self.0.channels,
      match self.0.buffer_size {
        BufferSize::Default => display("default"),
        BufferSize::Fixed(n) => display(n),
      }
    )
  }
}
