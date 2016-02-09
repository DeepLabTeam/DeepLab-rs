use std::rc::Rc;

use dl;
use graphics;
use matrix;
use piston::input;
use opengl_graphics::GlGraphics;

use super::dl_ui::Mouse;
use super::node::{Node, NodeAction, NodeResponse};
use super::op::Operation;
use super::var_store::{VarIndex, VarStore};

pub enum GraphAction {
    SelectNode(NodeId),
    SelectVariable(VarIndex),
}

pub struct GraphBuilder {
    pub graph: dl::Graph,
    pub vars: VarStore,
    dim_vars: Vec<usize>,
    nodes: Vec<Node>,
    edges: Vec<(NodeId, usize, NodeId, usize)>,
    node_action: Option<(NodeId, NodeAction)>,
}

impl GraphBuilder {
    pub fn new() -> Self {
        GraphBuilder {
            graph: dl::Graph::new(),
            vars: VarStore::new(),
            dim_vars: vec![],
            nodes: vec![],
            edges: vec![],
            node_action: None,
        }
    }

    pub fn add_node(&mut self, name: String, pos: [f64; 2], op: Rc<Operation>) {
        let num_in = op.num_inputs;
        let mut outs = Vec::with_capacity(op.num_outputs as usize);
        for _ in 0..op.num_outputs {
            outs.push(self.vars.add((1, 1)));
        }
        self.nodes.push(Node::new(name, pos, op, num_in, outs));
    }

    pub fn event(&mut self, event: &input::Event, mouse: &Mouse) -> Option<GraphAction> {
        let mut graph_action = None;

        let mut new_action: Option<(NodeId, NodeAction)> = None;
        for (i, node) in self.nodes.iter_mut().enumerate() {
            node.event(event, mouse);
            if let Some(action) = node.action {
                new_action = Some((NodeId(i), action));
                break;
            }
        }

        if let Some((old_node, old_action)) = self.node_action {
            if let Some((new_node, _new_action)) = new_action {
                if let Some(response) = old_action.happened_before(&_new_action,
                                                                   (old_node, new_node)) {
                    match response {
                        NodeResponse::Connect(send_node, send_index, recv_node, recv_index) => {
                            // A connection was made
                            let v = self.nodes[send_node.0].outputs[send_index];
                            self.nodes[recv_node.0].inputs[recv_index] = Some(v);
                            self.edges.push((send_node, send_index, recv_node, recv_index));
                            if recv_node == new_node {
                                println!(" -> ");
                            } else {
                                println!(" <- ");
                            }
                        },
                        NodeResponse::Select => {
                            // Select the node
                            graph_action = Some(GraphAction::SelectNode(new_node));
                        },
                        NodeResponse::SelectInput(i) => {
                            // Select one of the node's input's
                            if let Some(v) = self.nodes[new_node.0].inputs[i] {
                                graph_action = Some(GraphAction::SelectVariable(v));
                            }
                        },
                        NodeResponse::SelectOutput(i) => {
                            // Select one of the node's output's
                            let v = self.nodes[new_node.0].outputs[i];
                            graph_action = Some(GraphAction::SelectVariable(v));
                        },
                    }
                    new_action = None;
                }
            }
        }

        if let Some((_, _new_action)) = new_action {
            match _new_action {
                NodeAction::DropInput(_) => { new_action = None; },
                NodeAction::DropOutput(_) => { new_action = None; },
                _ => { },
            }
        }

        self.node_action = new_action;

        graph_action
    }

    pub fn draw(&self, c: &graphics::Context, gl: &mut GlGraphics) {
        use graphics::Line;

        for &(send_node, send_index, recv_node, recv_index) in &self.edges {
            let start_pos = send_node.get(self).get_output_pos(send_index);
            let end_pos = recv_node.get(self).get_input_pos(recv_index);

            Line::new([1.0, 0.0, 0.0, 1.0], 1.0).draw([start_pos[0], start_pos[1],
                                                  end_pos[0], end_pos[1]],
                                                 &c.draw_state, c.transform, gl);
        }

        for node in &self.nodes {
            node.draw(c, gl);
        }
    }

    pub fn gpu_build(&mut self, ctx: &matrix::Context) {
        self.vars.gpu_build(ctx, &mut self.graph);

        for node in &mut self.nodes {
            (node.op.build)(ctx, &mut self.graph, &mut self.vars, &node.inputs, &node.outputs);
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub struct NodeId(usize);

impl NodeId {
    pub fn get<'a>(&self, graph: &'a GraphBuilder) -> &'a Node {
        &graph.nodes[self.0]
    }
}

#[derive(Copy, Clone, PartialEq)]
pub struct DimVar(usize);

impl DimVar {
    pub fn get<'a>(&self, graph: &'a GraphBuilder) -> &'a usize {
        &graph.dim_vars[self.0]
    }
}
