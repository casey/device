use super::*;

pub(crate) struct Input {
  queue: Arc<Mutex<Vec<f32>>>,
  #[allow(unused)]
  stream: cpal::Stream,
  stream_config: StreamConfig,
}

impl Input {
  pub(crate) fn drain(&self) -> Sound {
    Sound {
      samples: std::mem::take(&mut self.queue.lock().unwrap()),
      channels: self.stream_config.channels,
      sample_rate: self.stream_config.sample_rate.0,
    }
  }

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
