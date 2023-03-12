use windows::core::Result;
use crate::{matrix_handler::MatrixHandler, options::Options};

#[derive(Copy, Clone, PartialEq)]
pub enum WorkingStatus {
    Working,
    Finished,
}

pub trait Stepper {
    fn next(&mut self, options: &Options, handler: &mut MatrixHandler) -> Result<WorkingStatus>;
}

pub struct NilStepper { }
impl Stepper for NilStepper {
    fn next(&mut self, _: &Options, _: &mut MatrixHandler) -> Result<WorkingStatus> {
        Ok(WorkingStatus::Finished)
    }
}

