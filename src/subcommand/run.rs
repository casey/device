use super::*;

#[derive(Default, Parser)]
pub(crate) struct Run {
  #[arg(long)]
  fullscreen: bool,
  #[arg(long)]
  present_mode: Option<PresentMode>,
  #[arg(long)]
  record: bool,
}

impl Run {
  pub(crate) fn run(self, options: Options, config: Config) -> Result {
    let record = self
      .record
      .then(|| options.fps.context(error::RecordRequiresFps))
      .transpose()?;

    let mut app = App::new(config, self.fullscreen, options, self.present_mode, record)?;

    let event_loop = EventLoop::with_user_event()
      .build()
      .context(error::EventLoopBuild)?;

    event_loop.set_control_flow(ControlFlow::Poll);

    event_loop.run_app(&mut app).context(error::AppRun)?;

    app.finish()?;

    Ok(())
  }
}
