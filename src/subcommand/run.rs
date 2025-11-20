use super::*;

pub(crate) fn run(options: Options) -> Result {
  let mut app = App::new(options)?;

  EventLoop::with_user_event()
    .build()
    .context(error::EventLoopBuild)?
    .run_app(&mut app)
    .context(error::AppRun)?;

  app.save_recording()?;

  app.errors()?;

  Ok(())
}
