use std::sync::mpsc::*;

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, PartialOrd)]
#[allow(dead_code)] // using any particular value is up to user
pub enum ThreadControl {
    Pause,
    Resume,
    Stop,
}

impl ThreadControl {
    pub fn make_channels() -> (Sender<ThreadControl>, Receiver<ThreadControl>) {
        channel::<ThreadControl>()
    }
}
