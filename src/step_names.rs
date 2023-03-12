use windows::core::Result;
use crate::{matrix_handler::MatrixHandler, options::Options, stepper::*};

pub struct NameList {}
impl Stepper for NameList {
    fn next(&mut self, options: &Options, handler: &mut MatrixHandler) -> Result<WorkingStatus> {
        let catcode = crate::continuum_preset::HCCategoryCode::new();
        let presets = handler.get_presets();
        if presets.is_empty() {
            println!("No user presets found");
        } else {
            for preset in presets.iter() {
                preset.print();
                preset.print_friendly_categories(&catcode);
            }
            crate::preset_listing::save_preset_listing(&presets[0..], options.get_path());
        }
        Ok(WorkingStatus::Finished)
    }

}