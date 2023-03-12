use windows::{core::*, Win32::Foundation::E_FAIL};
use crate::{
    continuum_preset::*,
    preset_listing::*,
    stepper::*,
    matrix_handler::MatrixHandler,
    options::*,
};

fn fail(message: &str) -> windows::core::Result<()> {
    Err(Error::new(E_FAIL, HSTRING::from(message)))
}

pub fn save_preset(options: &Options, handler: &mut MatrixHandler, preset: &ContinuumPreset) ->  windows::core::Result<()> {
    let data = handler.get_archive_data();
    if let Some(mut path) = options.get_path() {
        // if action is:
        //   SaveCurrent | filename in path is .mid file
        //   Save        | filename in path is listing file. Use file's folder.
        if path.is_dir() {
            path.push(make_preset_filename(&preset.name, &data));
        } else if options.action == Action::Save {
            match path.parent() {
                Some(parent) => {
                    path = parent.to_path_buf();
                    path.push(make_preset_filename(&preset.name, &data));
                }
                None => {
                    let message = format!(
                        "Couldn't save '{}' : Unable to get target folder",
                        path.display()
                    );
                    return fail(&message);
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
                fail(&error.to_string())
            }
        }
    } else {
        fail("Missing path to save to")
    }
}

pub struct SingleSaver { }
impl SingleSaver {
    fn first_handler_preset(handler: &MatrixHandler) -> Option<ContinuumPreset>
    {
        handler.get_presets().first().cloned()
    }
}
impl Stepper for SingleSaver {
    fn next(&mut self, options: &Options, handler: &mut MatrixHandler) -> Result<WorkingStatus> {
        if let Some(preset) = Self::first_handler_preset(handler) {
            _ = save_preset(options, handler, &preset);
        }
        Ok(WorkingStatus::Finished)
    }
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

pub struct Saver {
    save_state: SaveState,
    working_preset: u8,
    presets: Vec<ContinuumPreset>,
}

impl Saver {
    pub fn new() -> Self {
        Self {
            save_state: SaveState::default(),
            working_preset: u8::MAX,
            presets: Vec::new(),
        }
    }

    fn handle_error(error: Error) -> Result<WorkingStatus> {
        println!("Unable to continue due to error: {}", error.message());
        Ok(WorkingStatus::Finished)
    }

}

impl Stepper for Saver {
    fn next(&mut self, options: &Options, handler: &mut MatrixHandler) -> Result<WorkingStatus> {
        match self.save_state {
            SaveState::Start => {
                println!("Gathering user presets...");
                self.save_state = SaveState::GatherList;
                self.working_preset = 0;
                Ok(WorkingStatus::Working)
            }
            SaveState::GatherList => {
                debug_assert!(self.presets.is_empty());
                self.presets = handler.get_presets().clone();
                for preset in self.presets.iter() {
                    println!("{}", preset.name);
                }
                self.save_state = SaveState::CollectPreset;
                Ok(if self.presets.is_empty() {
                    WorkingStatus::Finished
                } else {
                    WorkingStatus::Working
                })
            }
            SaveState::CollectPreset => {
                debug_assert!((self.working_preset as usize) < self.presets.len());
                println!(
                    "Collecting {}...",
                    self.presets[self.working_preset as usize].name
                );
                let index = self.presets[self.working_preset as usize].number;
                //$review:error handling
                if let Err(error) = handler.choose_preset(index) {
                    return Self::handle_error(error);
                }
                if let Err(error) = handler.start_action(Action::SaveCurrent) {
                    return Self::handle_error(error);
                }
                self.save_state = SaveState::SavePreset;
                Ok(WorkingStatus::Working)
            }
            SaveState::SavePreset => {
                let preset = &self.presets[self.working_preset as usize];
                _ = save_preset(options, handler, preset); // $review: error handling
                self.working_preset += 1;
                if (self.working_preset as usize) >= self.presets.len() {
                    self.working_preset = u8::MAX;
                    self.save_state = SaveState::Finish;
                } else {
                    self.save_state = SaveState::CollectPreset;
                }
                Ok(WorkingStatus::Working)
            }
            SaveState::Finish => {
                save_preset_listing(&self.presets[0..], options.get_path());
                self.save_state = SaveState::Start;
                Ok(WorkingStatus::Finished)
            }
        }
    }
}