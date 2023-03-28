use crate::{
    continuum_preset::{ContinuumPreset, PresetBuilder},
    haken_midi::cc16,
    matrix_handler::{ArchiveState, MatrixHandler},
    midi::CHANNEL16,
    options::Options,
    preset_listing::*,
    read_midi_file::ReadMidiFile,
    stepper::*,
    util::is_extension,
};
use std::path::PathBuf;
use std::io::Write;
use windows::{core::*, Win32::Foundation::E_FAIL};

#[derive(Copy, Clone, Default, PartialEq)]
enum SendState {
    #[default]
    Start,
    Prologue,
    Matrix,
    Name,
    Save,
    Finish,
}

fn busy_wait(dt: &std::time::Duration) {
    if dt.is_zero() { return; }
    let start = std::time::Instant::now();
    loop {
        let elapsed = start.elapsed();
        if elapsed.is_zero() { break; }
        if elapsed > *dt { break; }
        std::thread::yield_now();
    }
}

pub struct PresetLoader {
    initialized: bool,
    index: usize,
    state: SendState,
    is_data_sent: bool,
    presets: Vec<ContinuumPreset>,
    folder: PathBuf,
}

impl PresetLoader {
    pub fn new() -> Self {
        Self {
            initialized: false,
            index: usize::MAX,
            state: SendState::default(),
            is_data_sent: false,
            presets: Vec::new(),
            folder: PathBuf::default(),
        }
    }

    fn fail(message: &str) -> Result<WorkingStatus> {
        Err(Error::new(E_FAIL, HSTRING::from(message)))
    }

    fn read_preset_folder(
        path: &PathBuf,
        presets: &mut Vec<ContinuumPreset>,
    ) -> Result<WorkingStatus> {
        let mut builder = PresetBuilder::default();
        match std::fs::read_dir(path) {
            Ok(paths) => {
                let mut index = 1;
                for path in paths.flatten() {
                    let filename = path.file_name();
                    let name = filename.to_string_lossy();
                    if name.ends_with(".mid") {
                        builder.add_name_chars(&name[0..name.len() - 4]);
                        builder.set_number(index); // any will do - renumbered below
                        if let Some(preset) = builder.finish() {
                            presets.push(preset);
                        }
                        index += 1;
                        if index > 128 { break; }
                    }
                }
            }
            Err(error) => {
                return Self::fail(&error.to_string());
            }
        }

        presets.sort_unstable_by_key(|preset| preset.name.clone()); // how to avoid clone?
        for (index, preset) in presets.iter_mut().enumerate() {
            preset.number = (index + 1) as u8;
        }

        Ok(WorkingStatus::Working)
    }

    fn choose_current_slot(&mut self, handler: &mut MatrixHandler) -> Result<()> {
        let slot = self.presets[self.index].number;
        if 0 == slot {
            handler.choose_edit_slot()?;
        } else {
            handler.choose_preset(slot)?;
        }
        Ok(())
    }

    fn set_current_slot(&mut self, handler: &mut MatrixHandler) -> Result<()> {
        let slot = self.presets[self.index].number;
        if 0 == slot {
            handler.set_edit_slot()?;
        } else {
            handler.set_slot(slot)?;
        }
        Ok(())
    }
}

impl Stepper for PresetLoader {
    fn next(&mut self, options: &Options, handler: &mut MatrixHandler) -> Result<WorkingStatus> {
        match self.state {
            SendState::Start => {
                if !self.initialized {
                    self.presets.clear();
                    if let Some(path) = options.get_path() {
                        if path.is_file() {
                            // set base folder
                            if let Some(parent) = path.parent() {
                                self.folder = parent.into();
                            } else {
                                return Self::fail(&format!(
                                    "Missing required path in {}",
                                    path.to_string_lossy()
                                ));
                            }

                            // either listing (.txt) file or (.mid) file
                            if is_extension(&path, "txt") {
                                read_preset_listing(
                                    &path,
                                    &mut self.presets,
                                )?;
                            } else if is_extension(&path, "mid") {
                                if let Some(name) = path.file_stem() {
                                    let mut builder = PresetBuilder::default();
                                    builder.add_name_chars(&name.to_string_lossy());
                                    builder.set_number(0);
                                    if let Some(preset) = builder.finish() {
                                        self.presets.push(preset);
                                    }
                                }
                            } else {
                                return Self::fail(&format!(
                                    "Path is not a folder, preset listing (.txt), or preset (.mid) file: '{}'",
                                    path.to_string_lossy()));
                            }
                        } else {
                            self.folder = path.clone();
                            let listing = self.folder.join("UserPresets.txt");
                            if listing.exists() {
                                println!("Using preset listing '{}'", listing.to_string_lossy());
                                read_preset_listing(
                                    &listing,
                                    &mut self.presets,
                                )?;
                            } else {
                                Self::read_preset_folder(&path, &mut self.presets)?;
                            }
                        }
                    } else {
                        unreachable!();
                    }
                    debug_assert!(!self.presets.is_empty());
                    self.index = self.presets.len() -1;
                    self.initialized = true;
                }

                println!(">Starting preset load");
                self.choose_current_slot(handler)?;
                handler.editor_present()?; // editor present
                self.state = SendState::Prologue;
                Ok(WorkingStatus::Working)
            }


            SendState::Prologue => {
                if handler.editor_reply() {
                    println!(">Preparing device to receive");
                    handler.clear_archive_state();
                    handler.send_cc(
                        CHANNEL16,
                        cc16::DownloadInfo,
                        cc16::DownloadInfo_RetrieveArchive,
                    )?;
                    self.state = SendState::Matrix;
                }
                else {
                    handler.not_ready();
                }
                Ok(WorkingStatus::Working)
            }

            SendState::Matrix => {
                if !self.is_data_sent {
                    let mut path = self.folder.clone();
                    let mid_name = self.presets[self.index].name.clone() + ".mid";
                    path.push(&mid_name);
                    println!(">Sending preset data '{}'", path.to_string_lossy());
                    match std::fs::read(&path) {
                        Ok(data) => {
                            let mut file = ReadMidiFile::new(&data);
                            while let Some((dt, midi)) = file.next()? {
                                // if dt.as_millis() < 18 {
                                //     dt = std::time::Duration::from_millis(18);
                                // }
                                busy_wait(&dt); // sleep apparently causes mpsc::channel to deadlock, so busy-wait
                                midi.send(handler.output_port())?;
                            }
                            self.is_data_sent = true;
                        }
                        Err(error) => {
                            return Self::fail(&error.to_string());
                        }
                    }
                }

                // receive archiveOk 109:5 or archiveFail 109:6
                match handler.archive_state() {
                    ArchiveState::Unknown => {
                        //spin until archive state is known
                    }

                    ArchiveState::Ok => {
                        self.state = SendState::Name;
                        self.is_data_sent = false;
                        handler.not_ready();
                    }

                    ArchiveState::Fail => {
                        //return Self::fail("Preset loading failed");
                        println!("Preset loading failed");
                        self.is_data_sent = false;
                        self.state = SendState::Finish;
                        handler.editor_present()?;
                        handler.not_ready();
                    }
                }
                Ok(WorkingStatus::Working)
            }

            SendState::Name => {
                let name = &self.presets[self.index].name;
                println!(">Sending \"{}\" to slot {}", name, self.presets[self.index].number);
                handler.clear_presets();
                handler.send_string(0, name)?;
                self.set_current_slot(handler)?;
                self.state = SendState::Save;
                handler.not_ready();
                Ok(WorkingStatus::Working)
            }

            SendState::Save => {
                println!(">Save to flash");
                handler.send_cc(
                    CHANNEL16,
                    cc16::DownloadControl,
                    cc16::DownloadControl_SaveToFlash,
                )?;

                self.state = SendState::Finish;
                handler.editor_present()?;
                handler.not_ready();
                Ok(WorkingStatus::Working)
            }

            SendState::Finish => {
                if handler.editor_reply() {
                    println!();
                    if 0 ==  self.index {
                        Ok(WorkingStatus::Finished)
                    } else {
                        busy_wait(& std::time::Duration::new(1,0));
                        self.index -= 1;
                        self.state = SendState::Start;
                        Ok(WorkingStatus::Working)
                    }
                } else {
                    // wait for editor reply
                    _ = std::io::stdout().write(&[b'.']);
                    _ = std::io::stdout().flush();
                    handler.not_ready();
                    Ok(WorkingStatus::Working)
                }
            }
        }
    }
}
