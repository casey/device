use super::*;

pub(crate) fn run(options: Options) -> Result {
  let mut app = App::new(options)?;

  let event_loop = EventLoop::with_user_event()
    .build()
    .context(error::EventLoopBuild)?;

  event_loop.set_control_flow(ControlFlow::Poll);

  event_loop.run_app(&mut app).context(error::AppRun)?;

  app.errors()?;

  Ok(())
}
