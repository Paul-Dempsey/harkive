use std::sync::mpsc::*;

use crate::{acquire_device::*, midi_handler::*, thread_control::*};
use windows::{core::*, Devices::Midi::*, Foundation::*, Win32::Foundation::E_FAIL};

pub struct MidiSource {
    tx: Sender<WinMidi>,
    rx: Receiver<ThreadControl>,
    in_port: InPortDescription,
}

impl MidiSource {
    pub fn new(tx: Sender<WinMidi>, rx: Receiver<ThreadControl>, input: InPortDescription) -> Self {
        Self {
            tx,
            rx,
            in_port: input,
        }
    }

    fn hook_events(&self) -> windows::core::Result<()> {
        let tx = self.tx.clone();
        self.in_port.port.MessageReceived(&TypedEventHandler::new(
            move |_, arg: &Option<MidiMessageReceivedEventArgs>| {
                if arg.is_some() {
                    let imsg = unsafe { arg.as_ref().unwrap_unchecked().Message()? };
                    match tx.send(concrete_message(&imsg)) {
                        Ok(_) => {}
                        Err(e) => {
                            let message = e.to_string();
                            println!("Channel tx failed: {message}");
                            return Err(windows::core::Error::new(E_FAIL, HSTRING::from(message)));
                        }
                    }
                };
                Ok(())
            },
        ))?;
        Ok(())
    }

    pub fn run(&self) -> windows::core::Result<()> {
        self.hook_events()?;
        _ = self.rx.recv(); // any signal stops thread
        _ = self.in_port.port.Close();
        Ok(())
    }
}
