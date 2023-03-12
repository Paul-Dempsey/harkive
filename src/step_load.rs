
use windows::{
    core::*,
    Win32::Foundation::{E_FAIL},
};
use crate::{
    matrix_handler::{ MatrixHandler, ArchiveState }, midi::CHANNEL16, haken_midi::cc16,
    options::Options,
    read_midi_file::ReadMidiFile,
    stepper::*,
};


#[derive(Copy, Clone, Default, PartialEq)]
enum SendState {
    #[default]
    Start,
    Prologue,
    Matrix,
    Name,
    Save,
    Finish
}

pub struct PresetLoader {
    slot: u8,
    file: ReadMidiFile,
    name: String,
    state: SendState,
}

impl PresetLoader {
    pub fn new(slot:u8, name: &str, data: &[u8]) -> Self {
        Self {
            slot,
            name: name.to_string(),
            file: ReadMidiFile::new(data),
            state: SendState::default(),
        }
    }

    fn fail(message:&str) -> Result<WorkingStatus>
    {
        Err(Error::new(E_FAIL, HSTRING::from(message)))
    }
}
impl Stepper for PresetLoader
{    
    fn next(&mut self, _: &Options, handler: &mut MatrixHandler) -> Result<WorkingStatus> {
        match self.state {
            SendState::Start => {
                println!(">>Starting preset load");
                if 0 == self.slot {
                    handler.choose_edit_slot()?;
                } else {
                    handler.choose_preset(self.slot)?;
                }
                handler.editor_present()?; // editor present
                self.state = SendState::Prologue;
                Ok(WorkingStatus::Working)
            },

            SendState::Prologue => {
                println!(">>Sending preset data");
                handler.clear_archive_state();
                handler.send_cc(CHANNEL16, cc16::DownloadInfo, cc16::DownloadInfo_RetrieveArchive)?;
                self.state = SendState::Matrix;
                Ok(WorkingStatus::Working)
            },

            SendState::Matrix => {
                while let Some((_dt, midi)) = self.file.next()? {
                    // Commented out because it seems that sleeping when a
                    // chennel message is sent will cause mpsc recv to deadlock.
                    // and we seem to be working ok without delays.
                    // if !dt.is_zero() {
                    //     std::thread::sleep(dt);
                    // }
                    midi.send(handler.output_port())?;
                }

                // receive archiveOk 109:5 or archiveFail 109:6
                match handler.archive_state() {
                    ArchiveState::Unknown => { 
                        //spin until archive state is known
                    }

                    ArchiveState::Ok => {
                        self.state = SendState::Name;
                        handler.not_ready();
                        return Ok(WorkingStatus::Working);
                    }

                    ArchiveState::Fail => {
                        return Self::fail("Preset loading failed");
                    }
                }
                Ok(WorkingStatus::Working)
            },

            SendState::Name => {
                println!(">>Sending \"{}\" to slot {}", self.name, self.slot);
                handler.clear_presets();
                handler.send_string(0, &self.name)?;
                if 0 == self.slot {
                    handler.set_edit_slot()?;
                } else {
                    handler.set_slot(self.slot)?;
                }
                self.state = SendState::Save;
                handler.not_ready();
                Ok(WorkingStatus::Working)
            }

            SendState::Save => {
                println!(">>Save to flash");
                handler.send_cc(CHANNEL16, cc16::DownloadControl, cc16::DownloadControl_SaveToFlash)?;

                self.state = SendState::Finish;
                handler.editor_present()?;
                handler.not_ready();
                Ok(WorkingStatus::Working)
            }
            SendState::Finish => {
                Ok(WorkingStatus::Finished)
            }
        }
    }
}
