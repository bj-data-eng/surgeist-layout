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
    mod future_consumer {
        fn borrow(node: &crate::NodeInput, tree: &impl crate::LayoutTree) {
            let _ = (node, tree.node_input());
        }
    }
}

fn main() {}
