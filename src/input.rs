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
      samples: mem::take(&mut self.queue.lock().unwrap()),
      channels: self.stream_config.channels,
      sample_rate: self.stream_config.sample_rate.0,
    }
  }

  pub(crate) fn new(
    device: cpal::Device,
    supported_stream_config: SupportedStreamConfig,
  ) -> Result<Self> {
    let mut stream_config = supported_stream_config.config();

    stream_config.buffer_size = match supported_stream_config.buffer_size() {
      SupportedBufferSize::Range { min, max } => {
        cpal::BufferSize::Fixed(DEFAULT_BUFFER_SIZE.clamp(*min, *max))
      }
      SupportedBufferSize::Unknown => cpal::BufferSize::Default,
    };

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
          eprintln!("input stream error: {err}");
        },
        None,
      )
      .context(error::AudioBuildInputStream)?;

    stream.play().context(error::AudioPlayStream)?;

    log::info!(
      "input stream opened: {}x{}x{}",
      stream_config.sample_rate.0,
      stream_config.channels,
      match stream_config.buffer_size {
        cpal::BufferSize::Default => display("default"),
        cpal::BufferSize::Fixed(n) => display(n),
      }
    );

    Ok(Self {
      queue,
      stream,
      stream_config,
    })
  }
}
