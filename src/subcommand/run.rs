use super::*;

#[derive(Default, Parser)]
pub(crate) struct Run {
  #[arg(long)]
  record: bool,
  #[arg(long)]
  present_mode: Option<PresentMode>,
}

impl Run {
  pub(crate) fn run(self, options: Options, config: Config) -> Result {
    let mut app = App::new(options, self.present_mode, self.record, config)?;

    let event_loop = EventLoop::with_user_event()
      .build()
      .context(error::EventLoopBuild)?;

    event_loop.set_control_flow(ControlFlow::Poll);

    event_loop.run_app(&mut app).context(error::AppRun)?;

    app.errors()?;

    Ok(())
  }
}
