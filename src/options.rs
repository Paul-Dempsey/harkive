use crate::util::count_leading;
use std::{env, path::*};

#[derive(Copy, Clone, PartialEq)]
#[allow(dead_code)]
pub enum Action {
    Nothing,
    Usage,
    Docs,
    Monitor,
    ListMidi,
    ListNames,
    SaveCurrent,
    Save,
    Load,
    Clear,
}

pub struct Options {
    pub action: Action,
    path: Option<PathBuf>,
    pub device: Option<String>,
}

impl Options {
    pub fn docs() {
        //------------------------------------------------------------------------------
        println!(
            r#"---------------
hem-archive

Load and save presets from any device with Haken Audio's EaganMatrix engine.
Cannot be used while the Haken editor is running.

hem-archive [--device <name>] <action> [<path>]

--device   (-d) The name of device to save/restore from.

The device name can be a partial name as long as it is sufficiently unique.
For example, '-d Mini' is often sufficient to find a ContinuuMini, even if
other EaganMatrix devices are connected. If no device name is given, the
first suitable device is used.

<action> is one of:

--input    (-i) Print list of connected MIDI devices.
--monitor  (-m) Log MIDI received from the selected device.
--clear    (-c) Clear all user presets from the device.
--print    (-p) Print list of user presets.
--edit     (-e) Save current editing slot.
--save     (-s) Save user presets from the device to <path>.
--load     (-l) Load user presets from <path> to the device.
--help     (-h, -?) Help. The short forms print short help.

Preset lists are similar to Haken Editor group lists.

<path> usage:

--input, --monitor, and --clear do not use <path>.

<path> is required to load or save. The folder of the path must exist on disk.

<path> can generally be either a file path or a folder. When no file name is
given, a default name is assumed or generated.

--save: When <path> is a file path, it is a preset list and the preset .mid
files go to the same folder. The default file name is "UserPresets.txt".

--edit: If <path> ends with <name>.mid, the editing slot midi data is written
to that filename. Otherwise, <path> is a folder. If the slot is unnamed or
"Empty", a unique filename is generated in the format "anon-NNNN.mid" using
a hash of the preset midi data.

--load: When <path> is a file name, it is either preset list (.txt), or a 
preset midi data (.mid) file. For a .mid file, the preset is loaded into slot
zero, the editing slot. For a preset list file, the preset .mid files are
expected in the same folder. The preset numbers are interpreted as absolute
preset slot numbers from 1 to 128. When <path> is a folder, if the folder
contains a UserPresets.txt file, that list file is used. Otherwise all preset
.mid files in the folder are loaded in alphabetical order.
"#
        );
    }

    pub fn usage() {
        println!(
            r#"hem-archive [--device <name>] <action> [<path>]

--device   (-d) Name of device to save/restore from.
--input    (-i) Print list of connected MIDI devices.
--monitor  (-m) Log MIDI received from the selected device.
--clear    (-c) Clear all user presets from the device.
--print    (-p) Print list of user presets.
--edit     (-e) Save current editing slot.
--save     (-s) Save user presets from the device to <path>.
--load     (-l) Load user presets from <path> to the device.
--help     (-h, -?) Print help info. The short forms print this summary info.
"#
        );
    }

    pub fn validate(&self) -> bool {
        match self.action {
            Action::Nothing
            | Action::Usage
            | Action::Docs
            | Action::ListMidi
            | Action::ListNames
            | Action::Monitor => true,

            Action::Save | Action::SaveCurrent | Action::Load => {
                if self.path.is_none() {
                    println!("Missing folder to save/restore to/from.");
                    false
                } else {
                    true
                }
            }
            Action::Clear => true,
        }
    }

    fn set_action(&mut self, act: Action) -> bool {
        match self.action {
            Action::Nothing | Action::Usage | Action::Docs => {
                self.action = act;
                true
            }
            _ => {
                println!("Only one action per run can be used.");
                false
            }
        }
    }

    pub fn get_options() -> Option<Self> {
        let mut options = Self::default();
        let mut expect_device = false;
        for arg in env::args_os().skip(1) {
            if let Ok(sarg) = arg.clone().into_string() {
                match &sarg[0..] {
                    "--help" | "--doc" | "--man" => {
                        options.action = Action::Docs;
                        return Some(options);
                    }
                    "-h" | "-?" => {
                        options.action = Action::Usage;
                        return Some(options);
                    }
                    "--print" | "-p" => {
                        if !options.set_action(Action::ListNames) {
                            return None;
                        }
                    }
                    "--monitor" | "-m" => {
                        if !options.set_action(Action::Monitor) {
                            return None;
                        }
                    }
                    "--input" | "-i" => {
                        if !options.set_action(Action::ListMidi) {
                            return None;
                        }
                    }
                    "--save" | "-s" => {
                        if !options.set_action(Action::Save) {
                            return None;
                        }
                    }
                    "--load" | "-l" => {
                        if !options.set_action(Action::Load) {
                            return None;
                        }
                    }
                    "--edit" | "-e" => {
                        if !options.set_action(Action::SaveCurrent) {
                            return None;
                        }
                    }
                    "--clear" | "-c" => {
                        if !options.set_action(Action::Clear) {
                            return None;
                        }
                    }
                    "--device" | "-d" => {
                        expect_device = true;
                    }
                    _ => {
                        if count_leading('-', &sarg[0..]) > 0 {
                            println!("Unknown option {sarg}");
                            return None;
                        } else {
                            if expect_device {
                                options.device = Some(arg.to_string_lossy().to_string());
                                expect_device = false;
                                continue;
                            }
                            let path = Path::new(&arg);
                            match path.canonicalize() {
                                Ok(path) => {
                                    options.path = Some(path);
                                }
                                Err(error) => {
                                    println!("Error: {error}");
                                    return None;
                                }
                            };
                        }
                    }
                }
            }
        }
        if options.validate() {
            Some(options)
        } else {
            None
        }
    }
    pub fn get_path(&self) -> Option<PathBuf> {
        self.path.as_ref().cloned()
    }
    pub fn get_path_display_name(&self) -> Option<String> {
        if let Some(path) = &self.path {
            if let Some(name) = path.to_str() {
                return Some(name.to_string());
            }
        }
        None
    }
}

impl Default for Options {
    fn default() -> Self {
        Self {
            action: Action::Usage,
            path: None,
            device: None,
        }
    }
}
