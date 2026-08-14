// normalize-stderr-test: "(\n)\n$" -> "$1"
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

mod block {
    mod consumer {
        fn direct_type(_: &crate::NodeInput) {}

        type HiddenCompleteInput = crate::NodeInput;
    }
}

mod inline {
    mod future {
        mod deeper {
            fn new_descendant(_: &crate::LayoutInput) {}
        }
    }
}

mod flex {
    pub(crate) use crate::NodeInput as VisibleCompleteInput;

    mod consumer {
        fn direct(tree: &impl crate::LayoutTree) {
            let _ = tree.node_input();
        }
    }
}

mod grid {
    mod input {
        type OwnerAliasEscape = crate::LayoutInput;
    }

    mod consumer {
        fn ufcs<T: crate::LayoutTree>(tree: &T) {
            let _ = crate::LayoutTree::node_input(tree);
        }
    }
}

mod scroll {
    mod input {
        pub(super) use crate::LayoutInput as OwnerVisibleEscape;
    }

    mod nested {
        mod consumer {
            fn extracted<T: crate::LayoutTree>() {
                let _lookup = <T as crate::LayoutTree>::node_input;
            }

            fn literals_and_comments_are_not_identity() {
                let _ordinary = "NodeInput LayoutInput node_input";
                let _raw = r#"NodeInputOf LayoutInputOf"#;
                // NodeInput LayoutInput node_input
                /* NodeInputOf LayoutInputOf */
            }

            fn direct_node_input_of(_: &crate::NodeInputOf) {}

            fn direct_layout_input_of(_: &crate::LayoutInputOf) {}
        }
    }
}

fn main() {}
