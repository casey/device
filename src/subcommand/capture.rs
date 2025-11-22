use super::*;

#[derive(Parser)]
pub(crate) struct Capture {
  #[arg(long)]
  pub(crate) duration: NonZeroU32,
}

impl Capture {
  pub(crate) fn run(self, options: Options) -> Result {
    let mut stream = options.stream()?;

    let recorder = Arc::new(Mutex::new(Recorder::new()?));

    let mut analyzer = Analyzer::new();

    let state = options.state();

    let resolution = state.resolution.unwrap_or(RESOLUTION);

    let mut renderer = pollster::block_on(Renderer::new(
      None,
      Vector2::new(resolution, resolution),
      resolution,
    ))?;

    let fps = state.fps.unwrap_or(FPS.try_into().unwrap());

    let spf = fps.spf(Synthesizer::SAMPLE_RATE)?;

    let (tx, rx) = mpsc::channel();

    let frames = u64::from(self.duration.get()) * u64::from(fps.fps().get());

    let progress = ProgressBar::new(frames);

    for frame in 0..frames {
      progress.inc(1);

      for _ in 0..spf * u32::from(Synthesizer::CHANNELS) {
        stream.next();
      }

      let sound = stream.drain();
      analyzer.update(&sound, false, &state);
      renderer.render(&analyzer, &state, Instant::now())?;

      let recorder = recorder.clone();
      let tx = tx.clone();
      renderer.capture(move |image| {
        if let Err(err) = tx.send(recorder.lock().unwrap().frame(frame, image, sound)) {
          eprintln!("failed to send captured frame: {err}");
        }
      })?;

      renderer.poll()?;

      rx.recv().unwrap()?;
    }

    progress.finish();

    recorder.lock().unwrap().save(&options)?;

    Ok(())
  }
}
