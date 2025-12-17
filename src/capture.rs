use super::*;

pub(crate) struct Capture {
  pub(crate) buffer: Buffer,
  pub(crate) bytes_per_row_with_padding: usize,
  pub(crate) callback: Box<dyn FnOnce(Image) + Send + 'static>,
  pub(crate) format: ImageFormat,
  pub(crate) pool: Arc<Mutex<Vec<Buffer>>>,
  pub(crate) size: Size,
}
