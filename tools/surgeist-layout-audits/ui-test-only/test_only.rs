#![deny(p01_i08_s02_r06_t02_node_projection_boundary)]

mod node_input {
    pub struct NodeInputOf;
    pub enum LayoutInputOf {
        Box(NodeInputOf),
    }
}

pub use node_input::{LayoutInputOf, NodeInputOf};
pub type NodeInput = NodeInputOf;

mod tree {
    pub trait LayoutTree {
        fn node_input(&self) -> &crate::NodeInputOf;
    }
}

pub use tree::LayoutTree;

mod block {
    #[cfg(test)]
    mod test_support {
        type Complete = crate::NodeInput;

        fn borrow(node: &crate::LayoutInputOf, tree: &impl crate::LayoutTree) {
            let _ = (node, tree.node_input());
        }
    }
}

fn main() {}
