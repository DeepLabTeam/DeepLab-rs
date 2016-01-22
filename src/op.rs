use dl;
use matrix;

use super::dl_ui::DeepLabUi;
use super::var_store::VarIndex;

pub struct Operation {
    pub name: String,
    pub num_inputs: u64,
    pub num_outputs: u64,
    pub build: Box<Fn(&DeepLabUi, &mut dl::Graph, &[Option<VarIndex>], &[Option<VarIndex>])>,
}

impl Operation {
    pub fn new(name: String, num_inputs: u64, num_outputs: u64,
               build: Box<Fn(&DeepLabUi, &mut dl::Graph, &[Option<VarIndex>], &[Option<VarIndex>])>)
               -> Self {
        Operation {
            name: name,
            num_inputs: num_inputs,
            num_outputs: num_outputs,
            build: build,
        }
    }
}
