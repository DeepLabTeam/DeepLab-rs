use dl;
use matrix;

#[derive(Copy, Clone)]
pub struct Variable {
    pub shape: (usize, usize),
    pub gpu: Option<dl::VarIndex>,
}

pub struct VarStore {
    vars: Vec<Variable>,
}

impl VarStore {
    pub fn new() -> Self {
        VarStore {
            vars: vec![],
        }
    }

    pub fn add(&mut self, shape: (usize, usize)) -> VarIndex {
        self.vars.push(Variable { shape: shape, gpu: None });
        VarIndex(self.vars.len()-1)
    }

    pub fn get(&self, v: VarIndex) -> &Variable {
        &self.vars[v.0]
    }

    pub fn get_mut(&mut self, v: VarIndex) -> &mut Variable {
        &mut self.vars[v.0]
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct VarIndex(usize);

impl VarIndex {
    pub fn get<'a>(&self, v: &'a VarStore) -> &'a Variable {
        v.get(*self)
    }

    pub fn get_mut<'a>(&self, v: &'a mut VarStore) -> &'a mut Variable {
        v.get_mut(*self)
    }
}
