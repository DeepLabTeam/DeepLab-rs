use matrix;
use dl;

use super::var_store::{VarIndex, VarStore};

pub type BuildFn = Box<Fn(&matrix::Context, &mut dl::Graph, &VarStore, &[Option<VarIndex>], &[VarIndex]) -> dl::NodeIndex>;

pub struct Operation {
    pub name: String,
    pub num_inputs: u64,
    pub num_outputs: u64,
    pub build: BuildFn,
}

impl Operation {
    pub fn new(name: String, num_inputs: u64, num_outputs: u64, build: BuildFn) -> Self {
        Operation {
            name: name,
            num_inputs: num_inputs,
            num_outputs: num_outputs,
            build: build,
        }
    }
}
