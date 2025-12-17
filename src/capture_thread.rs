use super::*;

pub(crate) struct CaptureThread {
  buffer_sender: mpsc::Sender<Capture>,
  join_handle: JoinHandle<()>,
}

impl CaptureThread {
  fn capture(capture: Capture) {
    let Capture {
      buffer,
      bytes_per_row_with_padding,
      callback,
      captures,
      format,
      size,
    } = capture;

    let view = buffer.get_mapped_range(..);

    let bytes_per_row = size.x.get().into_usize() * COLOR_CHANNELS;

    let mut image = Image::default();
    image.resize(size.x.get(), size.y.get());
    for (src, dst) in view
      .chunks(bytes_per_row_with_padding)
      .map(|src| &src[..bytes_per_row])
      .take(size.y.get().into_usize())
      .zip(image.data_mut().chunks_mut(bytes_per_row))
    {
      for (src, dst) in src
        .chunks(COLOR_CHANNELS)
        .zip(dst.chunks_mut(COLOR_CHANNELS))
      {
        format.swizzle(src.try_into().unwrap(), dst.try_into().unwrap());
      }
    }

    drop(view);

    buffer.unmap();

    captures.lock().unwrap().push(buffer);

    callback(image);
  }

  pub(crate) fn finish(self) -> Result {
    drop(self.buffer_sender);

    match self.join_handle.join() {
      Ok(()) => Ok(()),
      Err(panic_value) => Err(error::RecordingJoin { panic_value }.build()),
    }
  }

  pub(crate) fn new() -> Result<Self> {
    let (buffer_sender, rx) = mpsc::channel();

    let join_handle = thread_spawn("capture", move || {
      loop {
        let Ok(capture) = rx.recv() else {
          break;
        };

        Self::capture(capture);
      }
    })?;

    Ok(Self {
      buffer_sender,
      join_handle,
    })
  }

  pub(crate) fn tx(&self) -> &mpsc::Sender<Capture> {
    &self.buffer_sender
  }
}
