// normalize-stderr-test: "(\n)\n$" -> "$1"
// compile-flags: --edition=2024
#![deny(p01_i08_s02_r06_t02_node_projection_boundary)]

mod node_input {
    pub struct NodeInputOf;
}

pub use node_input::NodeInputOf;
pub type NodeInput = NodeInputOf;

mod tree {
    pub trait LayoutTree {
        fn node_input(&self) -> &crate::NodeInputOf;
    }
}

pub use tree::LayoutTree;

fn complete_input() -> NodeInput {
    NodeInputOf
}

mod grid {
    mod input {
        #[macro_export]
        macro_rules! owner_expression {
            ($tree:expr) => {
                $tree.node_input()
            };
        }

        #[macro_export]
        macro_rules! caller_expression {
            ($input:expr) => {
                $input
            };
        }

        #[macro_export]
        macro_rules! consumer_items {
            () => {
                fn generated_type(_: &crate::NodeInput) {}

                type GeneratedAlias = crate::NodeInput;

                pub(super) use crate::NodeInput as GeneratedVisibleInput;
            };
        }

        pub(crate) use crate::caller_expression;
        pub(crate) use crate::consumer_items;
        pub(crate) use crate::owner_expression;
    }

    mod consumer {
        super::input::consumer_items!();

        fn macro_expressions(tree: &impl crate::LayoutTree) {
            let _ = super::input::owner_expression!(tree);
            let _ = super::input::caller_expression!(crate::complete_input());
        }
    }
}

fn main() {}
