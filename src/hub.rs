use super::*;

pub(crate) struct Hub {
  #[allow(unused)]
  connections: Vec<midir::MidiInputConnection<()>>,
  messages: Arc<Mutex<Vec<Message>>>,
}

impl Hub {
  pub(crate) fn messages(&mut self) -> &Mutex<Vec<Message>> {
    &self.messages
  }

  pub(crate) fn new() -> Result<Self> {
    let messages = Arc::new(Mutex::new(Vec::new()));

    let mut connections = Vec::new();

    let input = midir::MidiInput::new("MIDI Input").context(error::MidiInputInit)?;
    for port in input.ports() {
      let name = input.port_name(&port).context(error::MidiPortInfo)?;
      let messages = messages.clone();
      connections.push(
        midir::MidiInput::new(&format!("MIDI Port Input: {name}"))
          .context(error::MidiInputInit)?
          .connect(
            &port,
            &name,
            move |_timestamp, event, ()| match Message::parse(event) {
              Ok(message) => messages.lock().unwrap().push(message),
              Err(err) => log::warn!("MIDI event parse error: {err}"),
            },
            (),
          )
          .context(error::MidiInputPortConnect)?,
      );
    }

    Ok(Self {
      connections,
      messages,
    })
  }
}
