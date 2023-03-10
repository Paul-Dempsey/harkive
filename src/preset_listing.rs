use crate::continuum_preset::*;
use std::{
    fs::File,
    io::{self, BufRead},
    path::*,
};
use windows::{core::*, Win32::Foundation::E_FAIL};

pub fn save_preset_listing(presets: &[ContinuumPreset], folder: Option<PathBuf>) {
    if let Some(folder) = folder {
        let mut text = String::new();
        for preset in presets.iter().rev() {
            text += &format!("{},\"{}.mid\"\n", 1 + preset.number, preset.name);
        }
        let mut path = folder;
        path.push("UserPresets.txt");
        match std::fs::write(&path, text) {
            Ok(_) => {
                println!("Saved preset list: '{}'", path.to_string_lossy());
            }
            Err(error) => {
                println!(
                    "Unable to save preset list '{}': {}",
                    path.to_string_lossy(),
                    error
                );
            }
        }
    }
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

// Matches HakenEditor behavior where encountering an invalid line
// silently stops parsing.
pub fn read_preset_listing(path: &PathBuf) -> Result<Vec<ContinuumPreset>> {
    let mut result: Vec<ContinuumPreset> = Vec::new();
    let mut builder = PresetBuilder::default();
    match read_lines(path) {
        Ok(lines) => {
            let string_trim: &[char] = &[' ', '\t', '"'];
            for line in lines {
                match line {
                    Ok(line) => {
                        builder.start();
                        let mut pieces = line.split(',');
                        if let Some(mut s) = pieces.next() {
                            s = s.trim();
                            match s.parse::<u8>() {
                                Ok(n) => {
                                    builder.set_number(n);
                                }
                                Err(_) => break,
                            }
                        } else {
                            break;
                        }
                        if let Some(mut s) = pieces.next() {
                            s = s.trim_matches(string_trim);
                            builder.add_name_chars(s);
                            if let Some(preset) = builder.finish() {
                                result.push(preset);
                            } else {
                                unreachable!();
                            }
                        } else {
                            break;
                        }
                    }
                    Err(_) => {
                        break;
                    }
                }
            }
        }
        Err(error) => return Err(Error::new(E_FAIL, HSTRING::from(error.to_string()))),
    }
    Ok(result)
}
