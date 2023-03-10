use crate::{
    acquire_device::*,
    matrix_handler::MatrixHandler,
    midi_source::MidiSource,
    options::{Action, Options},
    preset_listing::*,
    preset_loader::PresetLoader,
    thread_control::*,
};
use crate::{continuum_preset::*, midi::is_midi_header, midi_handler::*, util::*};
use std::sync::mpsc::*;
use windows::{core::*, Win32::Foundation::E_FAIL};

#[derive(Copy, Clone, PartialEq)]
pub enum WorkingStatus {
    Working,
    Finished,
}

#[derive(Copy, Clone, Default, PartialEq)]
enum SaveState {
    #[default]
    Start,
    GatherList,
    CollectPreset,
    SavePreset,
    Finish,
}

pub struct PresetManager<'a> {
    options: &'a Options,
    input: InPortDescription,
    handler: MatrixHandler,
    presets: Vec<ContinuumPreset>,
    save_state: SaveState,
    working_preset: u8,
    loader: Option<PresetLoader>,
}

impl<'a> PresetManager<'a> {
    pub async fn new(options: &'a Options) -> Option<PresetManager<'a>> {
        if let Some((input, output)) = get_haken_io(&options.device).await {
            Some(PresetManager {
                options,
                input,
                handler: MatrixHandler::new(output),
                presets: Vec::new(),
                working_preset: u8::MAX,
                save_state: SaveState::default(),
                loader: None,
            })
        } else {
            println!("Unable to find a suitable available device.");
            None
        }
    }

    fn fail(message: &str) -> windows::core::Result<()> {
        Err(Error::new(E_FAIL, HSTRING::from(message)))
    }

    fn handle_error(error: Error) -> WorkingStatus {
        println!("Unable to continue due to error: {}", error.message());
        WorkingStatus::Finished
    }

    fn start_action(&mut self) -> windows::core::Result<()> {
        match self.options.action {
            Action::Nothing | Action::Usage | Action::Docs | Action::Monitor | Action::ListMidi => {
                unreachable!()
            }

            Action::ListNames => {}
            Action::SaveCurrent => {}
            Action::Save => {
                println!("Gathering user presets...");
                self.save_state = SaveState::GatherList;
                self.working_preset = 0;
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
                                    self.loader = Some(PresetLoader::new(&name, &data[0..]));
                                } else {
                                    todo!()
                                }
                            }
                            Err(error) => {
                                return Self::fail(&error.to_string());
                            }
                        }
                    } else {
                        todo!();
                    }
                }
            }
            Action::Clear => {}
        };
        self.handler.start_action(self.options.action)
    }

    fn finish_action(&mut self) -> WorkingStatus {
        match self.options.action {
            Action::Nothing | Action::Usage | Action::Docs | Action::Monitor | Action::ListMidi => {
                WorkingStatus::Finished
            }

            Action::ListNames => {
                let catcode = HCCategoryCode::new();
                let presets = self.handler.get_presets();
                if presets.is_empty() {
                    println!("No user presets found");
                } else {
                    for preset in presets.iter() {
                        preset.print();
                        preset.print_friendly_categories(&catcode);
                    }
                    save_preset_listing(&presets[0..], self.options.get_path());
                }
                WorkingStatus::Finished
            }

            Action::SaveCurrent => {
                if let Some(preset) = self.first_handler_preset() {
                    _ = self.save_preset(&preset);
                }
                WorkingStatus::Finished
            }

            Action::Save => self.finish_save(),
            Action::Load => self.finish_load(),
            Action::Clear => WorkingStatus::Finished,
        }
    }

    fn first_handler_preset(&self) -> Option<ContinuumPreset> {
        self.handler.get_presets().first().cloned()
    }

    fn finish_save(&mut self) -> WorkingStatus {
        match self.save_state {
            SaveState::Start => unreachable!(),
            SaveState::GatherList => {
                debug_assert!(self.presets.is_empty());
                self.presets = self.handler.get_presets().clone();
                for preset in self.presets.iter() {
                    println!("{}", preset.name);
                }
                self.save_state = SaveState::CollectPreset;
                if self.presets.is_empty() {
                    WorkingStatus::Finished
                } else {
                    WorkingStatus::Working
                }
            }
            SaveState::CollectPreset => {
                debug_assert!((self.working_preset as usize) < self.presets.len());
                println!(
                    "Collecting {}...",
                    self.presets[self.working_preset as usize].name
                );
                let index = self.presets[self.working_preset as usize].number;
                //$review:error handling
                if let Err(error) = self.handler.choose_preset(index) {
                    return Self::handle_error(error);
                }
                if let Err(error) = self.handler.start_action(Action::SaveCurrent) {
                    return Self::handle_error(error);
                }
                self.save_state = SaveState::SavePreset;
                WorkingStatus::Working
            }
            SaveState::SavePreset => {
                let preset = self.presets[self.working_preset as usize].clone();
                _ = self.save_preset(&preset); // $review: error handling
                self.working_preset += 1;
                if (self.working_preset as usize) >= self.presets.len() {
                    self.working_preset = u8::MAX;
                    self.save_state = SaveState::Finish;
                } else {
                    self.save_state = SaveState::CollectPreset;
                }
                WorkingStatus::Working
            }
            SaveState::Finish => {
                save_preset_listing(&self.presets[0..], self.options.get_path());
                self.save_state = SaveState::Start;
                WorkingStatus::Finished
            }
        }
    }

    fn finish_load(&mut self) -> WorkingStatus {
        if let Some(loader) = &mut self.loader {
            match loader.next(&mut self.handler) {
                Ok(status) => status,
                Err(error) => {
                    println!("{error}");
                    self.loader = None;
                    WorkingStatus::Finished
                }
            }
        } else {
            WorkingStatus::Finished
        }
    }

    fn make_preset_filename(preset: &ContinuumPreset, data: &[u8]) -> String {
        let anon = is_empty_preset_name(&preset.name);
        if anon {
            println!("Renaming Empty or un-named preset");
        }
        (if anon {
            format!("anon-{}", short_hash(data))
        } else {
            preset.name.clone()
        } + ".mid")
    }

    fn save_preset(&mut self, preset: &ContinuumPreset) -> windows::core::Result<()> {
        let data = self.handler.get_archive_data();
        if let Some(mut path) = self.options.get_path() {
            // if action is:
            //   SaveCurrent | filename in path is .mid file
            //   Save        | filename in path is listing file. Use file's folder.
            if path.is_dir() {
                path.push(Self::make_preset_filename(preset, &data));
            } else if self.options.action == Action::Save {
                match path.parent() {
                    Some(parent) => {
                        path = parent.to_path_buf();
                        path.push(Self::make_preset_filename(preset, &data));
                    }
                    None => {
                        let message = format!(
                            "Couldn't save '{}' : Unable to get target folder",
                            path.display()
                        );
                        return Self::fail(&message);
                    }
                }
            }
            let pathname = path.display().to_string();
            match std::fs::write(path, &data) {
                Ok(_) => {
                    println!("Saved preset '{pathname}'");
                    Ok(())
                }
                Err(error) => {
                    println!("Couldn't save '{pathname}' : {error}");
                    Self::fail(&error.to_string())
                }
            }
        } else {
            unreachable!();
        }
    }

    fn handle_midi(&mut self, msg: &WinMidi, thread_tx: &Sender<ThreadControl>) -> bool {
        if dispatch_midi(&mut self.handler, msg).is_err()
            || (self.handler.is_ready() && (WorkingStatus::Finished == self.finish_action()))
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
                            && (WorkingStatus::Finished == self.finish_action())
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
