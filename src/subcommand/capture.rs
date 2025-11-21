use super::*;

// todo:
// - progress bar

pub(crate) fn run(options: Options) -> Result {
  let mut synthesizer = Synthesizer::busy_signal();

  let recorder = Arc::new(Mutex::new(Recorder::new()?));

  let mut analyzer = Analyzer::new();

  let resolution = 1024.try_into().unwrap();

  let state = Program::Highwaystar.state().db(-10);

  let mut renderer = pollster::block_on(Renderer::new(
    None,
    Vector2::new(resolution, resolution),
    resolution,
  ))?;

  let (tx, rx) = mpsc::channel();

  for i in 0..120 {
    eprintln!("rendering frame {i}");

    for _ in 0..800 * Synthesizer::CHANNELS {
      synthesizer.next();
    }

    let sound = synthesizer.drain();
    analyzer.update(&sound, false, &state);

    renderer.render(&analyzer, &state, Instant::now())?;

    let recorder = recorder.clone();
    let tx = tx.clone();

    renderer.capture(move |image| {
      if let Err(err) = tx.send(recorder.lock().unwrap().frame(i, image, sound)) {
        eprintln!("failed to send captured frame: {err}");
      }
    })?;

    renderer.poll()?;

    rx.recv().unwrap()?;
  }

  recorder.lock().unwrap().save(&options)?;

  Ok(())
}
