use std::sync::mpsc::*;
use windows::{core::*, Win32::Foundation::E_FAIL};
use crate::{
    acquire_device::*,
    matrix_handler::MatrixHandler,
    midi_handler::*,
    midi_source::MidiSource,
    midi::is_midi_header,
    options::{Action, Options},
    step_load::PresetLoader,
    stepper::*,
    step_names::NameList,
    step_save::*,
    thread_control::*,
};

pub struct PresetManager<'a> {
    options: &'a Options,
    input: InPortDescription,
    handler: MatrixHandler,
    stepper: Box<dyn Stepper>,
}

impl<'a> PresetManager<'a> {
    pub async fn new(options: &'a Options) -> Option<PresetManager<'a>> {
        if let Some((input, output)) = get_haken_io(&options.device).await {
            Some(PresetManager {
                options,
                input,
                handler: MatrixHandler::new(output),
                stepper: Box::new(NilStepper{}),
            })
        } else {
            println!("Unable to find a suitable available device.");
            None
        }
    }

    fn fail(message: &str) -> windows::core::Result<()> {
        Err(Error::new(E_FAIL, HSTRING::from(message)))
    }

    fn start_action(&mut self) -> windows::core::Result<()> {
        match self.options.action {
            Action::ListNames => {
                self.stepper = Box::new(NameList{});
            }
            Action::SaveCurrent => {
                self.stepper = Box::new(SingleSaver{});
            }
            Action::Save => {
                self.stepper = Box::new(Saver::new())
            }
            Action::Load => {
                if let Some(path) = self.options.get_path() {
                    if path.is_file() {
                        // either listing (.txt) file or (.mid) file
                        match std::fs::read(&path) {
                            Ok(data) => {
                                if is_midi_header(&data[0..]) {
                                    let name = if let Some(filename) = &path.file_stem() {
                                        filename.to_string_lossy().to_string()
                                    } else {
                                        "Empty".to_string()
                                    };
                                    self.stepper = Box::new(PresetLoader::new(0, &name, &data[0..]));
                                } else {
                                    //let presets = crate::preset_listing::read_preset_listing(&path);
                                }
                            }
                            Err(error) => {
                                return Self::fail(&error.to_string());
                            }
                        }
                    } else {
                        // folder - enumerate first 128 presets in alphabetical order
                        todo!();
                    }
                }
            }
            _ => {}
        };
        self.handler.start_action(self.options.action)
    }

    fn step_action(&mut self) -> WorkingStatus {
        match self.stepper.next(self.options, &mut self.handler) {
            Ok(status) => status,
            Err(error) => {
                println!("{error}");
                WorkingStatus::Finished
            }
        }
    }

    fn handle_midi(&mut self, msg: &WinMidi, thread_tx: &Sender<ThreadControl>) -> bool {
        if dispatch_midi(&mut self.handler, msg).is_err()
            || (self.handler.is_ready() && (WorkingStatus::Finished == self.step_action()))
        {
            _ = thread_tx.send(ThreadControl::Stop);
            return false;
        }
        true
    }

    pub fn run(&mut self) -> windows::core::Result<()> {
        let (midi_tx, midi_rx) = channel::<WinMidi>();
        let (thread_tx, thread_rx) = ThreadControl::make_channels();
        let midi_source = MidiSource::new(midi_tx, thread_rx, self.input.clone());
        let joiner = std::thread::spawn(move || midi_source.run());

        self.start_action()?;
        std::thread::sleep(std::time::Duration::from_millis(100));

        loop {
            match midi_rx.try_recv() {
                // pump ready messages
                Ok(msg) => {
                    if !self.handle_midi(&msg, &thread_tx) {
                        break;
                    }
                }
                // otherwise process idle, then block for next midi message
                Err(e) => match e {
                    TryRecvError::Empty => {
                        // tell handler we're idle (no pending messages)
                        self.handler.on_idle();
                        // handle finish state as needed
                        if self.handler.is_ready()
                            && (WorkingStatus::Finished == self.step_action())
                        {
                            _ = thread_tx.send(ThreadControl::Stop);
                            break;
                        }
                        // wait for next midi message
                        match midi_rx.recv() {
                            Ok(msg) => {
                                if !self.handle_midi(&msg, &thread_tx) {
                                    break;
                                }
                            }
                            Err(_) => break,
                        }
                    }
                    TryRecvError::Disconnected => break,
                },
            }
        }
        if let Err(error) = joiner.join() {
            println!("Thread join error: {error:?}");
        }
        Ok(())
    }
}
