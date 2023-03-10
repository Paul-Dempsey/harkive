
use windows::{
    core::*,
    Win32::Foundation::{E_FAIL},
};
use crate::{
    read_midi_file::ReadMidiFile,
    preset_manager::WorkingStatus,
    matrix_handler::{ MatrixHandler, ArchiveState },
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
    file: ReadMidiFile,
    name: String,
    state: SendState,
}

impl PresetLoader {
    pub fn new(name: &str, data: &[u8]) -> Self {
        Self {
            name: name.to_string(),
            file: ReadMidiFile::new(data),
            state: SendState::default(),
        }
    }

    fn fail(message:&str) -> Result<WorkingStatus>
    {
        Err(Error::new(E_FAIL, HSTRING::from(message)))
    }
    
    pub fn next(&mut self, handler: &mut MatrixHandler) -> Result<WorkingStatus> {
        match self.state {
            SendState::Start => {
                println!(">>Starting preset load");
                handler.choose_edit_slot()?;
                handler.editor_present()?; // editor present
                self.state = SendState::Prologue;
                Ok(WorkingStatus::Working)
            },
            SendState::Prologue => {
                println!(">>Sending preset data");
                handler.clear_archive_state();
                handler.send_cc(15, 110, 121)?; // Retrieve Archive
                self.state = SendState::Matrix;
//                handler.unready();
                Ok(WorkingStatus::Working)
            },
            SendState::Matrix => {
                while let Some((_dt, midi)) = self.file.next()? {
                    // Commented out because it seems that sleeping when a
                    // chennel message is sent will cause mpsc recv to deadlock.
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
                        handler.unready();
                        return Ok(WorkingStatus::Working);
                    }

                    ArchiveState::Fail => {
                        return Self::fail("Preset loading failed");
                    }
                }
                Ok(WorkingStatus::Working)
            },
            SendState::Name => {
                println!(">>Sending \"{}\" and slot", self.name);
                handler.clear_presets();
                handler.send_string(0, &self.name)?;
                handler.set_edit_slot()?;

                self.state = SendState::Save;
                handler.unready();
                Ok(WorkingStatus::Working)
            }
            SendState::Save => {
                println!(">>Save to flash");
                handler.send_cc(15, 109, 8)?; // save to flash

                self.state = SendState::Finish;
                handler.editor_present()?;
                handler.unready();
                Ok(WorkingStatus::Working)
            }
            SendState::Finish => {
                Ok(WorkingStatus::Finished)
            }
        }
    }
}
