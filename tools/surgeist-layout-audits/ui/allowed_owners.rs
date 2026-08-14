#![deny(p01_i08_s02_r06_t02_node_projection_boundary)]

mod node_input {
    pub struct NodeInputOf;
    pub enum LayoutInputOf {
        Box(NodeInputOf),
    }
}

pub use node_input::{LayoutInputOf, NodeInputOf};
pub type NodeInput = NodeInputOf;
pub type LayoutInput = LayoutInputOf;

mod tree {
    pub trait LayoutTree {
        fn node_input(&self) -> &crate::NodeInputOf;
    }
}

pub use tree::LayoutTree;

mod node_projection {
    fn borrow(
        node: &crate::NodeInput,
        layout: &crate::LayoutInput,
        tree: &impl crate::LayoutTree,
    ) {
        let _ = (node, layout, tree.node_input());
    }
}

macro_rules! allowed_input_owner {
    ($module:ident) => {
        mod $module {
            mod input {
                fn borrow(
                    node: &crate::NodeInputOf,
                    layout: &crate::LayoutInputOf,
                    tree: &impl crate::LayoutTree,
                ) {
                    let _ = (node, layout, tree.node_input());
                }
            }
        }
    };
}

allowed_input_owner!(block);
allowed_input_owner!(inline);
allowed_input_owner!(flex);
allowed_input_owner!(grid);
allowed_input_owner!(scroll);

fn main() {}
