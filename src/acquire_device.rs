use crate::{midi_traits::Named, util::edit_distance};
use windows::{core::*, Devices::Enumeration::*, Devices::Midi::*};

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Direction {
    In,
    Out,
}
impl Named for Direction {
    fn name(&self) -> &'static str {
        match *self {
            Direction::In => "in",
            Direction::Out => "out",
        }
    }
}
impl Direction {
    pub fn selector(&self) -> Option<HSTRING> {
        if let Ok(selector) = match self {
            Direction::In => MidiInPort::GetDeviceSelector(),
            Direction::Out => MidiOutPort::GetDeviceSelector(),
        } {
            Some(selector)
        } else {
            None
        }
    }
}

pub fn trim_port_tag(name: &str) -> &str {
    name.trim_end_matches(|ch| matches!(ch, '[' | ']' | '0'..='9'))
        .trim_end()
}

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum HakenDeviceKind {
    Continuum,
    ContinuuMini,
    EaganMatrixModule,
    Osmose,
    NotHakenDevice,
}
impl HakenDeviceKind {
    fn identify(name: &str) -> HakenDeviceKind {
        if name.len() >= 6 {
            if name.contains("Continuum") {
                HakenDeviceKind::Continuum
            } else if name.contains("ContinuuMini") {
                HakenDeviceKind::ContinuuMini
            } else if name.contains("EaganMatrix") {
                // probably "EaganMatrixModule", but prefix should sufficient (it's used internally)).
                HakenDeviceKind::EaganMatrixModule
            } else if name.contains("Osmose") {
                HakenDeviceKind::Osmose
            } else {
                HakenDeviceKind::NotHakenDevice
            }
        } else {
            HakenDeviceKind::NotHakenDevice
        }
    }
}
impl Named for HakenDeviceKind {
    fn name(&self) -> &'static str {
        match self {
            HakenDeviceKind::Continuum => "Continuum",
            HakenDeviceKind::ContinuuMini => "ContinuuMini",
            HakenDeviceKind::EaganMatrixModule => "EaganMatrix Module",
            HakenDeviceKind::Osmose => "Osmose",
            HakenDeviceKind::NotHakenDevice => "Not a Haken device",
        }
    }
}

#[derive(Clone)]
pub struct DeviceDescriptor {
    pub direction: Direction,
    pub kind: HakenDeviceKind,
    pub name: HSTRING,
    pub id: HSTRING,
}
impl DeviceDescriptor {
    pub fn friendly_name(&self) -> String {
        let name = self.name.to_string_lossy();
        trim_port_tag(&name).to_string()
    }
}

pub async fn list_midi_devices() {
    async fn list_devices(label: &str, direction: Direction) {
        if let Some(devices) = get_info_collection(direction).await {
            for item in devices.into_iter() {
                if let Ok(hname) = item.Name() {
                    let name = hname.to_string_lossy();
                    println!("{label}: {name}");
                }
            }
        }
    }
    list_devices(" in", Direction::In).await;
    list_devices("out", Direction::Out).await;
}

pub async fn get_first_haken_device(direction: Direction) -> Option<DeviceDescriptor> {
    if let Some(devices) = get_info_collection(direction).await {
        for item in devices.into_iter() {
            if let Ok(hname) = item.Name() {
                let name = hname.to_string_lossy();
                let name = trim_port_tag(&name);
                let kind = HakenDeviceKind::identify(name);
                if kind != HakenDeviceKind::NotHakenDevice {
                    if let Ok(id) = item.Id() {
                        return Some(DeviceDescriptor {
                            direction,
                            kind,
                            name: hname,
                            id,
                        });
                    }
                }
            }
        }
    }
    None
}

pub async fn get_info_collection(direction: Direction) -> Option<DeviceInformationCollection> {
    if let Some(selector) = direction.selector() {
        if let Ok(filter) = DeviceInformation::FindAllAsyncAqsFilter(&selector) {
            if let Ok(devices) = filter.await {
                return Some(devices);
            }
        }
    }
    None
}

pub async fn get_haken_device(direction: Direction, device_name: &str) -> Option<DeviceDescriptor> {
    struct DeviceMatch {
        kind: HakenDeviceKind,
        name: HSTRING,
        id: HSTRING,
        score: u32,
    }
    fn rank_name(name: &str) -> String {
        let name = trim_port_tag(name);
        let mut result = String::with_capacity(name.len());
        let mut leading = true;
        let mut is_space = false;
        for ch in name.chars() {
            match ch {
                ' ' => {
                    if !leading && !is_space {
                        result.push(ch);
                        is_space = true;
                    }
                }
                '0'..='9' => {
                    if !leading {
                        result.push(ch);
                    }
                    is_space = false;
                }
                'a'..='z' | 'A'..='Z' => {
                    leading = false;
                    is_space = false;
                    result.push(ch);
                }
                _ => {}
            }
        }
        // pad to mitigate large scores due to length mismatches
        for _ in result.len()..=32 {
            result.push('~');
        }
        result
    }

    let mut candidates = Vec::<DeviceMatch>::default();
    if let Some(devices) = get_info_collection(direction).await {
        for item in devices.into_iter() {
            if let Ok(hname) = item.Name() {
                if let Ok(id) = item.Id() {
                    let name = rank_name(&hname.to_string_lossy());
                    let kind = HakenDeviceKind::identify(&name);
                    if kind != HakenDeviceKind::NotHakenDevice {
                        let score = edit_distance(device_name, &name)
                            - if name.contains(device_name) { 5 } else { 0 };
                        //println!("Checking Device: {} score={}/{} name={}", name, score, name.contains(device_name), hname.to_string_lossy());
                        candidates.push(DeviceMatch {
                            kind,
                            name: hname,
                            id,
                            score,
                        });
                    }
                }
            }
        }
    }
    if !candidates.is_empty() {
        let mut score = u32::MAX;
        let mut least = usize::MAX;
        for (index, device) in candidates.iter().enumerate() {
            if device.score < score {
                score = device.score;
                least = index;
            }
        }
        if score != u32::MAX {
            let d = &candidates[least];
            return Some(DeviceDescriptor {
                direction,
                kind: d.kind,
                name: d.name.clone(),
                id: d.id.clone(),
            });
        }
    }
    None
}

#[derive(Clone)]
pub struct InPortDescription {
    pub port: MidiInPort,
    pub description: DeviceDescriptor,
}
pub struct OutPortDescription {
    pub port: MidiOutPort,
    pub description: DeviceDescriptor,
}

async fn open_in_port(info: &DeviceDescriptor) -> Option<MidiInPort> {
    debug_assert!(info.direction == Direction::In);
    if let Ok(future) = MidiInPort::FromIdAsync(&info.id) {
        match future.await {
            Ok(port) => {
                return Some(port);
            }
            Err(ref error) => {
                if HRESULT(0) == error.code() {
                    println!(
                        "Midi in port '{}' is in use ({})",
                        info.friendly_name(),
                        error.message()
                    );
                } else {
                    println!(
                        "Error opening Midi in port '{}': {}",
                        info.friendly_name(),
                        error.message()
                    );
                }
            }
        }
    }
    None
}

async fn open_out_port(info: &DeviceDescriptor) -> Option<MidiOutPort> {
    debug_assert!(info.direction == Direction::Out);
    if let Ok(future) = MidiOutPort::FromIdAsync(&info.id) {
        match future.await {
            Ok(iport) => {
                if let Ok(port) = iport.cast() {
                    return Some(port);
                }
            }
            Err(ref error) => {
                if HRESULT(0) == error.code() {
                    println!("Midi out port '{}' is in use", info.friendly_name());
                } else {
                    println!(
                        "Error opening Midi out port '{}': {}",
                        info.friendly_name(),
                        error.message()
                    );
                }
            }
        }
    }
    None
}

pub async fn get_haken_io(
    device_name: &Option<String>,
) -> Option<(InPortDescription, OutPortDescription)> {
    if let Some(in_device) = if let Some(name) = device_name {
        get_haken_device(Direction::In, name).await
    } else {
        get_first_haken_device(Direction::In).await
    } {
        if let Some(out_device) = get_haken_device(Direction::Out, &in_device.friendly_name()).await
        {
            if let Some(in_port) = open_in_port(&in_device).await {
                if let Some(out_port) = open_out_port(&out_device).await {
                    return Some((
                        InPortDescription {
                            port: in_port,
                            description: in_device,
                        },
                        OutPortDescription {
                            port: out_port,
                            description: out_device,
                        },
                    ));
                }
            }
        }
    }
    None
}
