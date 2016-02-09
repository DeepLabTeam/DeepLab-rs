use matrix;
use dl;

use super::var_store::{VarIndex, VarStore};

//pub trait BuildFn : Fn(&matrix::Context, &mut dl::Graph, &VarStore, &[Option<VarIndex>], &[VarIndex]) -> dl::NodeIndex { }

pub struct Operation {
    pub name: String,
    pub num_inputs: usize,
    pub num_outputs: usize,
    pub build: Box<Fn(&matrix::Context, &mut dl::Graph, &mut VarStore, &[Option<VarIndex>], &[VarIndex])>,
}

impl Operation {
    pub fn new<F>(name: String, num_inputs: usize, num_outputs: usize, build: F) -> Self
        where F: Fn(&matrix::Context, &mut dl::Graph, &mut VarStore,
                    &[Option<VarIndex>], &[VarIndex]) + 'static {
        Operation {
            name: name,
            num_inputs: num_inputs,
            num_outputs: num_outputs,
            build: Box::new(build),
        }
    }
}
