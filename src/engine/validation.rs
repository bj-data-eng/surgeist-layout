use crate::{
    AtomicInlineParticipationRoleError, FloatExclusion, FloatExclusionRoleError, LayoutErrorKindOf,
    LayoutErrorOf, LayoutErrorSiteOf, LayoutInputOf, LayoutInvalidInputOf, LayoutOperation,
    LayoutResultOf, LayoutScalar, LayoutTree, NodeInputOf, NonBoxNodeRoleError, Position,
};
use crate::{Display, LayoutUnsupportedCapability};

pub(crate) fn validate_layout_request<Tree>(
    tree: &Tree,
    root: Tree::Node,
    changed_nodes: &[Tree::Node],
) -> LayoutResultOf<Tree::Node, Vec<Tree::Node>, Tree::Scalar, Tree::MeasureError>
where
    Tree: LayoutTree,
{
    let invalidated_nodes = invalidation_closure(tree, root, changed_nodes)?;
    if validate_layout_tree(tree, root)? {
        return Err(LayoutErrorOf::new(
            LayoutErrorSiteOf::Node(root),
            LayoutOperation::RootLayout,
            LayoutErrorKindOf::UnsupportedCapability(LayoutUnsupportedCapability::LaterFriBehavior),
        ));
    }
    Ok(invalidated_nodes)
}

fn invalidation_closure<Tree>(
    tree: &Tree,
    root: Tree::Node,
    changed_nodes: &[Tree::Node],
) -> LayoutResultOf<Tree::Node, Vec<Tree::Node>, Tree::Scalar, Tree::MeasureError>
where
    Tree: LayoutTree,
{
    #[derive(Clone, Copy, Eq, PartialEq)]
    enum VisitState {
        Visiting,
        Complete,
    }

    struct DiscoveredNode<Node> {
        node: Node,
        parents: Vec<usize>,
        state: VisitState,
    }

    struct Frame<Node> {
        node_index: usize,
        children: Vec<Node>,
        next_child: usize,
    }

    let mut discovered = vec![DiscoveredNode {
        node: root,
        parents: Vec::new(),
        state: VisitState::Visiting,
    }];
    let mut path = vec![Frame {
        node_index: 0,
        children: tree.children(root).collect(),
        next_child: 0,
    }];

    while !path.is_empty() {
        let current = path.len() - 1;
        if path[current].next_child == path[current].children.len() {
            discovered[path[current].node_index].state = VisitState::Complete;
            path.pop();
            continue;
        }

        let node_index = path[current].node_index;
        let node = discovered[node_index].node;
        let child = path[current].children[path[current].next_child];
        path[current].next_child += 1;

        if let Some(child_index) = discovered
            .iter()
            .position(|discovered| discovered.node == child)
        {
            if discovered[child_index].state == VisitState::Visiting {
                return Err(LayoutErrorOf::new(
                    LayoutErrorSiteOf::ContainerSubject {
                        container: node,
                        subject: child,
                    },
                    LayoutOperation::CacheInvalidation,
                    LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::TreeTopologyCycle),
                ));
            }
            discovered[child_index].parents.push(node_index);
            continue;
        }

        let child_index = discovered.len();
        discovered.push(DiscoveredNode {
            node: child,
            parents: vec![node_index],
            state: VisitState::Visiting,
        });
        path.push(Frame {
            node_index: child_index,
            children: tree.children(child).collect(),
            next_child: 0,
        });
    }

    let mut dirty_indices = Vec::new();
    for subject in changed_nodes.iter().copied() {
        let Some(index) = discovered
            .iter()
            .position(|discovered| discovered.node == subject)
        else {
            return Err(LayoutErrorOf::new(
                LayoutErrorSiteOf::Node(subject),
                LayoutOperation::CacheInvalidation,
                LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::InvalidationNodeNotReachable),
            ));
        };
        dirty_indices.push(index);
    }

    let mut included = vec![false; discovered.len()];
    while let Some(index) = dirty_indices.pop() {
        if included[index] {
            continue;
        }
        included[index] = true;
        dirty_indices.extend(discovered[index].parents.iter().copied());
    }

    Ok(discovered
        .into_iter()
        .zip(included)
        .filter_map(|(discovered, included)| included.then_some(discovered.node))
        .collect())
}

fn validate_layout_tree<Tree>(
    tree: &Tree,
    root: Tree::Node,
) -> LayoutResultOf<Tree::Node, bool, Tree::Scalar, Tree::MeasureError>
where
    Tree: LayoutTree,
{
    fn visit<Tree>(
        tree: &Tree,
        node: Tree::Node,
        is_root: bool,
        parent_accepts_inline_text: bool,
        later_behavior: &mut bool,
    ) -> LayoutResultOf<Tree::Node, (), Tree::Scalar, Tree::MeasureError>
    where
        Tree: LayoutTree,
    {
        let layout_input = tree.layout_input(node);
        let accepts_inline_text = match layout_input {
            LayoutInputOf::Box(_) => {
                let input = tree.node_input(node);
                match (
                    !is_root && input.display.is_inline_level(),
                    input.atomic_inline_participation.is_some(),
                ) {
                    (true, false) => {
                        return Err(root_input_error(
                            node,
                            LayoutInvalidInputOf::AtomicInlineParticipation {
                                reason: AtomicInlineParticipationRoleError::MissingForAtomicInline,
                            },
                        ));
                    }
                    (false, true) => {
                        return Err(root_input_error(
                            node,
                            LayoutInvalidInputOf::AtomicInlineParticipation {
                                reason: AtomicInlineParticipationRoleError::UnexpectedForNonAtomic,
                            },
                        ));
                    }
                    (true, true) | (false, false) => {}
                }

                if input.float_exclusion == FloatExclusion::Shape {
                    let reason = if input.display == Display::None {
                        Some(FloatExclusionRoleError::Hidden)
                    } else if input.position == Position::Absolute {
                        Some(FloatExclusionRoleError::Absolute)
                    } else if input.float == crate::Float::None {
                        Some(FloatExclusionRoleError::NonFloating)
                    } else {
                        None
                    };
                    if let Some(reason) = reason {
                        return Err(root_input_error(
                            node,
                            LayoutInvalidInputOf::FloatExclusionRole { reason },
                        ));
                    }
                }
                input.display == Display::None || input.display.inner_display() == Display::Block
            }
            LayoutInputOf::InlineText(_) => {
                if let Some(reason) = non_box_node_role_error(tree, node) {
                    return Err(root_input_error(
                        node,
                        LayoutInvalidInputOf::NonBoxNodeRole { reason },
                    ));
                }
                if !parent_accepts_inline_text {
                    *later_behavior = true;
                }
                return Ok(());
            }
            LayoutInputOf::LineBreak(_) | LayoutInputOf::InlineBoundary(_) => {
                if let Some(reason) = non_box_node_role_error(tree, node) {
                    return Err(root_input_error(
                        node,
                        LayoutInvalidInputOf::NonBoxNodeRole { reason },
                    ));
                }
                return Ok(());
            }
        };

        for child in tree.children(node) {
            visit(tree, child, false, accepts_inline_text, later_behavior)?;
        }
        Ok(())
    }

    let mut later_behavior = false;
    visit(tree, root, true, false, &mut later_behavior)?;
    Ok(later_behavior)
}

fn non_box_node_role_error<Tree>(tree: &Tree, node: Tree::Node) -> Option<NonBoxNodeRoleError>
where
    Tree: LayoutTree,
{
    if tree.node_input(node) != &NodeInputOf::non_box() {
        Some(NonBoxNodeRoleError::NonCanonicalNodeInput)
    } else if tree.child_count(node) != 0 {
        Some(NonBoxNodeRoleError::HasChildren)
    } else if tree.has_leaf_measurement(node) {
        Some(NonBoxNodeRoleError::HasLeafMeasurement)
    } else {
        None
    }
}

fn root_input_error<Node, S, M>(
    node: Node,
    invalid: LayoutInvalidInputOf<S>,
) -> LayoutErrorOf<Node, S, M>
where
    S: LayoutScalar,
{
    LayoutErrorOf::new(
        LayoutErrorSiteOf::Node(node),
        LayoutOperation::RootLayout,
        LayoutErrorKindOf::InvalidInput(invalid),
    )
}
