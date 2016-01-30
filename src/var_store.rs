use dl;
use matrix;

#[derive(Copy, Clone)]
pub struct Variable {
    pub shape: (u64, u64),
    pub gpu: Option<dl::VarIndex>,
    managed: bool,
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

    pub fn add(&mut self, shape: (u64, u64)) -> VarIndex {
        self.vars.push(Variable { shape: shape, gpu: None, managed: false });
        VarIndex(self.vars.len()-1)
    }

    pub fn add_managed(&mut self, shape: (u64, u64)) -> VarIndex {
        self.vars.push(Variable { shape: shape, gpu: None, managed: true });
        VarIndex(self.vars.len()-1)
    }

    pub fn get(&self, v: VarIndex) -> &Variable {
        &self.vars[v.0]
    }

    pub fn get_mut(&mut self, v: VarIndex) -> &mut Variable {
        &mut self.vars[v.0]
    }

    pub fn gpu_build(&mut self, ctx: &matrix::Context, graph: &mut dl::Graph) {
        for var in &mut self.vars {
            if var.managed {
                var.gpu = Some(graph.add_variable(ctx, var.shape));
            }
        }
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
