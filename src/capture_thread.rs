use super::*;

pub(crate) struct CaptureThread {
  handle: JoinHandle<()>,
  tx: mpsc::Sender<Capture>,
}

impl CaptureThread {
  fn capture(capture: Capture) {
    let view = capture.buffer.get_mapped_range(..);

    let bytes_per_row = capture.size.x.get().into_usize() * COLOR_CHANNELS;

    let mut image = Image::default();
    image.resize(capture.size.x.get(), capture.size.y.get());
    for (src, dst) in view
      .chunks(capture.bytes_per_row_with_padding)
      .map(|src| &src[..bytes_per_row])
      .take(capture.size.y.get().into_usize())
      .zip(image.data_mut().chunks_mut(bytes_per_row))
    {
      for (src, dst) in src
        .chunks(COLOR_CHANNELS)
        .zip(dst.chunks_mut(COLOR_CHANNELS))
      {
        capture
          .format
          .swizzle(src.try_into().unwrap(), dst.try_into().unwrap());
      }
    }

    drop(view);

    capture.buffer.unmap();

    capture.pool.lock().unwrap().push(capture.buffer);

    (capture.callback)(image);
  }

  pub(crate) fn finish(self) -> Result {
    drop(self.tx);

    match self.handle.join() {
      Ok(()) => Ok(()),
      Err(panic_value) => Err(error::RecordingJoin { panic_value }.build()),
    }
  }

  pub(crate) fn new() -> Result<Self> {
    let (tx, rx) = mpsc::channel();

    let handle = thread_spawn("capture", move || {
      loop {
        let Ok(capture) = rx.recv() else {
          break;
        };

        Self::capture(capture);
      }
    })?;

    Ok(Self { handle, tx })
  }

  pub(crate) fn tx(&self) -> &mpsc::Sender<Capture> {
    &self.tx
  }
}
