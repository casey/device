use super::*;

pub(crate) struct Input {
  queue: Arc<Mutex<Vec<f32>>>,
  #[allow(unused)]
  stream: cpal::Stream,
  stream_config: StreamConfig,
}

impl Input {
  pub(crate) fn new(device: rodio::Device, stream_config: SupportedStreamConfig) -> Result<Self> {
    // todo:
    // - is this actually doing what I think it's doing?
    // - use 128 buffer size and put it in a constant
    let buffer_size = match stream_config.buffer_size() {
      SupportedBufferSize::Range { min, .. } => {
        log::info!("input audio buffer size: {min}");
        Some(*min)
      }
      SupportedBufferSize::Unknown => {
        log::info!("input audio buffer size: unknown");
        None
      }
    };

    let mut stream_config = stream_config.config();

    if let Some(buffer_size) = buffer_size {
      stream_config.buffer_size = cpal::BufferSize::Fixed(buffer_size);
    }

    let queue = Arc::new(Mutex::new(Vec::new()));

    let stream = device
      .build_input_stream(
        &stream_config,
        {
          let queue = queue.clone();
          move |data: &[f32], _: &cpal::InputCallbackInfo| {
            queue.lock().unwrap().extend(data);
          }
        },
        move |err| {
          eprintln!("audio input error: {err}");
        },
        None,
      )
      .context(error::AudioBuildInputStream)?;

    stream.play().context(error::AudioPlayStream)?;

    Ok(Self {
      queue,
      stream,
      stream_config,
    })
  }
}

impl Stream for Input {
  fn append(&self, _sink: &Sink) {}

  fn channels(&self) -> u16 {
    self.stream_config.channels
  }

  fn drain_samples(&mut self, samples: &mut Vec<f32>) {
    samples.extend(self.queue.lock().unwrap().drain(..));
  }

  fn is_done(&self) -> bool {
    false
  }

  fn sample_rate(&self) -> u32 {
    self.stream_config.sample_rate.0
  }
}

impl Iterator for Input {
  type Item = f32;

  fn next(&mut self) -> Option<f32> {
    Some(0.0)
  }
}
