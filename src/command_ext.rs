use {super::*, std::process::Command};

pub(crate) trait CommandExt {
  fn stdout_utf8(&mut self) -> Result<String>;
}

impl CommandExt for Command {
  fn stdout_utf8(&mut self) -> Result<String> {
    let output = self.output().context(error::CommandRun {
      program: self.get_program(),
    })?;

    ensure! {
      output.status.success(),
      error::CommandStatus { status: output.status, stderr: output.stderr, program: self.get_program() },
    }

    String::from_utf8(output.stdout).context(error::CommandUtf8 {
      program: self.get_program(),
    })
  }
}
