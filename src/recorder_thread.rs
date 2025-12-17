use super::*;

pub(crate) struct RecorderThread {
  frame_sender: mpsc::Sender<(u64, Image, Sound)>,
  join_handle: JoinHandle<Result<Recorder>>,
}

impl RecorderThread {
  pub(crate) fn finish(self, options: &Options, config: &Config) -> Result {
    drop(self.frame_sender);

    match self.join_handle.join() {
      Ok(Ok(recorder)) => recorder.finish(options, config),
      Ok(Err(err)) => Err(err),
      Err(panic_value) => Err(error::RecordingJoin { panic_value }.build()),
    }
  }

  pub(crate) fn is_finished(&self) -> bool {
    self.join_handle.is_finished()
  }

  pub(crate) fn new(mut recorder: Recorder) -> Result<Self> {
    let (frame_sender, rx) = mpsc::channel::<(u64, Image, Sound)>();

    let join_handle = thread_spawn("recorder", move || {
      loop {
        let Ok((frame, image, sound)) = rx.recv() else {
          break;
        };

        recorder.frame(frame, image, sound)?;
      }

      Ok(recorder)
    })?;

    Ok(Self {
      frame_sender,
      join_handle,
    })
  }

  pub(crate) fn tx(&self) -> &mpsc::Sender<(u64, Image, Sound)> {
    &self.frame_sender
  }
}
