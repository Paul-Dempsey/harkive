use std::{sync::mpsc::*, thread};

use windows::{
    core::*,
    Devices::Midi::*,
    Win32::UI::Input::KeyboardAndMouse::{
        GetAsyncKeyState, VK_ESCAPE, VK_LCONTROL, VK_LSHIFT, VK_RCONTROL, VK_RSHIFT, VK_SPACE,
    },
};

mod acquire_device;
mod cc_text;
#[allow(dead_code)]
mod continuum_preset;
#[allow(dead_code)]
mod data_kind;
mod gather_state;
mod haken_midi;
mod matrix_handler;
#[allow(dead_code)]
mod midi;
mod midi_file;
mod midi_handler;
mod midi_monitor;
mod midi_source;
#[allow(dead_code)]
mod midi_traits;
mod options;
#[allow(dead_code)]
mod preset_listing;
mod step_load;
mod preset_manager;
mod read_midi_file;
mod stepper;
mod step_names;
mod step_save;
mod thread_control;
#[allow(dead_code)]
mod util;

use acquire_device::*;
use midi_handler::*;
use midi_monitor::MidiMonitor;
use midi_source::MidiSource;
use options::{Action, Options};
use thread_control::*;

fn main() -> Result<()> {
    futures::executor::block_on(main_async())
}

fn is_quit_key_pressed() -> bool {
    0 != unsafe {
        GetAsyncKeyState(VK_LCONTROL.0 as i32)
            | GetAsyncKeyState(VK_RCONTROL.0 as i32)
            | GetAsyncKeyState(VK_LSHIFT.0 as i32)
            | GetAsyncKeyState(VK_RSHIFT.0 as i32)
            | GetAsyncKeyState(VK_SPACE.0 as i32)
            | GetAsyncKeyState(VK_ESCAPE.0 as i32)
    }
}

async fn midi_monitor(options: &Options) -> Result<()> {
    fn send_cc(port: &MidiOutPort, channel: u8, cc: u8, value: u8) -> Result<()> {
        port.SendMessage(&MidiControlChangeMessage::CreateMidiControlChangeMessage(
            channel, cc, value,
        )?)
    }

    println!("Monitoring MIDI.\nPress any of (SPACE, CTRL, ESC) then a note to stop.");
    if let Some((input, output)) = get_haken_io(&options.device).await {
        println!("Using {}", input.description.friendly_name());
        let (midi_tx, midi_rx) = channel::<WinMidi>();
        let (thread_tx, thread_rx) = ThreadControl::make_channels();
        let joiner = thread::spawn(move || {
            let midi_source = MidiSource::new(midi_tx, thread_rx, input);
            midi_source.run()
        });
        let mut handler = MidiMonitor::default();
        println!("[Enabling detailed MIDI output]");
        send_cc(&output.port, 15, 116, 85)?; // editor present
        println!("[Request User preset names]");
        send_cc(&output.port, 15, 109, 32)?; // send names
                                             //send_cc(&output.port,15, 109, 39)?; // sys names
        println!("[Request updates when presets change]");
        send_cc(&output.port, 15, 55, 1)?; // send updates
        let mut last = std::time::SystemTime::now();
        while let Ok(msg) = midi_rx.recv() {
            if is_quit_key_pressed() {
                _ = thread_tx.send(ThreadControl::Stop);
            } else {
                dispatch_midi(&mut handler, &msg)?;
                let now = std::time::SystemTime::now();
                if let Ok(dt) = now.duration_since(last) {
                    if dt.as_millis() > 1_000 {
                        println!("[Poll device status, DSP]");
                        send_cc(&output.port, 15, 116, 85)?; // editor present
                    }
                }
                last = now;
            }
        }
        match joiner.join() {
            Ok(_) => {}
            Err(error) => {
                println!("Error {error:?}");
            }
        }
    } else {
        println!("Unable to find a suitable device");
    }
    Ok(())
}

async fn main_async() -> Result<()> {
    if let Some(options) = Options::get_options() {
        match options.action {
            Action::Nothing | Action::Usage => {
                Options::usage();
            }
            Action::Docs => {
                Options::docs();
            }
            Action::ListMidi => {
                println!("MIDI devices:");
                list_midi_devices().await;
            }
            Action::ListNames | Action::Clear => {
                if let Some(mut manager) = preset_manager::PresetManager::new(&options).await {
                    manager.run()?;
                }
            }
            Action::Monitor => {
                midi_monitor(&options).await?;
            }
            Action::SaveCurrent | Action::Save | Action::Load => {
                let act = match options.action {
                    Action::SaveCurrent => "Save edit",
                    Action::Save => "Save",
                    Action::Load => "Load",
                    _ => unreachable!(),
                };
                if let Some(folder) = options.get_path_display_name() {
                    if let Some(device) = &options.device {
                        println!("{act} preset for {device} with {folder}");
                    } else {
                        println!("{act} preset with {folder}");
                    }
                } else {
                    unreachable!();
                }
                if let Some(mut manager) = preset_manager::PresetManager::new(&options).await {
                    manager.run()?;
                }
            }
        }
    } else {
        Options::usage();
    }
    Ok(())
}
