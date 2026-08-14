#![forbid(unsafe_code)]
#![feature(rustc_private)]

extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_middle;
extern crate rustc_span;

use rustc_hir::{
    AmbigArg, Expr, ExprKind, HirId, Item, ItemKind, Node, QPath, Ty, TyKind,
    def::{DefKind, Res},
};
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_middle::ty;
use rustc_span::{Span, Symbol, sym};

dylint_linting::declare_late_lint! {
    pub P01_I08_S02_R06_T02_NODE_PROJECTION_BOUNDARY,
    Allow,
    "complete node input escapes a projection-construction owner"
}

const AUDITED_TREES: &[&str] = &["block", "inline", "flex", "grid", "scroll"];
const INPUT_OWNERS: &[&[&str]] = &[
    &["block", "input"],
    &["inline", "input"],
    &["flex", "input"],
    &["grid", "input"],
    &["scroll", "input"],
];

impl<'tcx> LateLintPass<'tcx> for P01I08S02R06T02NodeProjectionBoundary {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx Item<'tcx>) {
        if is_test_only(cx, item.hir_id()) || !is_catalog_scope(cx, item.hir_id()) {
            return;
        }

        match item.kind {
            ItemKind::TyAlias(_, _, ty) if ty_is_protected(cx, ty) => {
                emit(cx, item.span, "complete node input alias escapes its owner");
            }
            ItemKind::Use(path, _) if !item.vis_span.is_empty() => {
                if path
                    .res
                    .present_items()
                    .any(|res| matches!(res, Res::Def(_, def_id) if def_is_protected(cx, def_id)))
                {
                    emit(
                        cx,
                        item.span,
                        "complete node input visibility reexport escapes its owner",
                    );
                }
            }
            _ => {}
        }
    }

    fn check_ty(&mut self, cx: &LateContext<'tcx>, ty: &'tcx Ty<'tcx, AmbigArg>) {
        if is_test_only(cx, ty.hir_id)
            || is_within_type_alias(cx, ty.hir_id)
            || !is_audited_tree(cx, ty.hir_id)
            || is_projection_owner(cx, ty.hir_id)
        {
            return;
        }

        if ty_is_protected(cx, ty.as_unambig_ty()) {
            emit(
                cx,
                ty.span,
                "complete node input type is used outside a projection-construction owner",
            );
        }
    }

    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx Expr<'tcx>) {
        if is_test_only(cx, expr.hir_id)
            || !is_audited_tree(cx, expr.hir_id)
            || is_projection_owner(cx, expr.hir_id)
        {
            return;
        }

        match expr.kind {
            ExprKind::MethodCall(..) => {
                if cx
                    .typeck_results()
                    .type_dependent_def_id(expr.hir_id)
                    .is_some_and(|def_id| is_layout_tree_node_input(cx, def_id))
                {
                    emit(
                        cx,
                        expr.span,
                        "LayoutTree::node_input is used outside a projection-construction owner",
                    );
                    return;
                }
            }
            ExprKind::Path(ref qpath) => {
                if qpath_def_id(cx, qpath, expr.hir_id)
                    .is_some_and(|def_id| is_layout_tree_node_input(cx, def_id))
                {
                    emit(
                        cx,
                        expr.span,
                        "LayoutTree::node_input is used outside a projection-construction owner",
                    );
                    return;
                }
            }
            ExprKind::Call(callee, _) if matches!(callee.kind, ExprKind::Path(ref qpath) if qpath_def_id(cx, qpath, callee.hir_id).is_some_and(|def_id| is_layout_tree_node_input(cx, def_id))) =>
            {
                return;
            }
            _ => {}
        }

        let expr_ty = cx.typeck_results().expr_ty_adjusted(expr);
        if semantic_ty_is_protected(cx, expr_ty) {
            emit(
                cx,
                expr.span,
                "complete node input value is used outside a projection-construction owner",
            );
        }
    }
}

fn emit(cx: &LateContext<'_>, span: Span, message: &'static str) {
    cx.emit_span_lint(
        P01_I08_S02_R06_T02_NODE_PROJECTION_BOUNDARY,
        span,
        rustc_errors::DiagDecorator(|diag| {
            diag.primary_message(message);
        }),
    );
}

fn qpath_def_id(
    cx: &LateContext<'_>,
    qpath: &QPath<'_>,
    hir_id: HirId,
) -> Option<rustc_hir::def_id::DefId> {
    match cx.qpath_res(qpath, hir_id) {
        Res::Def(_, def_id) => Some(def_id),
        _ => None,
    }
}

fn ty_is_protected(cx: &LateContext<'_>, hir_ty: &Ty<'_>) -> bool {
    match hir_ty.kind {
        TyKind::Path(ref qpath) => qpath_def_id(cx, qpath, hir_ty.hir_id)
            .is_some_and(|def_id| def_is_protected(cx, def_id)),
        _ => false,
    }
}

fn semantic_ty_is_protected(cx: &LateContext<'_>, ty: ty::Ty<'_>) -> bool {
    match ty.peel_refs().kind() {
        ty::Adt(adt, _) => is_protected_aggregate_def(cx, adt.did()),
        _ => false,
    }
}

fn def_is_protected(cx: &LateContext<'_>, def_id: rustc_hir::def_id::DefId) -> bool {
    if is_protected_aggregate_def(cx, def_id) {
        return true;
    }

    if cx.tcx.def_kind(def_id) == DefKind::TyAlias {
        let alias_ty = cx
            .tcx
            .type_of(def_id)
            .instantiate_identity()
            .skip_norm_wip();
        if semantic_ty_is_protected(cx, alias_ty) {
            return true;
        }
    }

    let mut ancestor = def_id;
    while let Some(parent) = cx.tcx.opt_parent(ancestor) {
        if is_protected_aggregate_def(cx, parent) {
            return true;
        }
        ancestor = parent;
    }
    false
}

fn is_protected_aggregate_def(cx: &LateContext<'_>, def_id: rustc_hir::def_id::DefId) -> bool {
    matches!(cx.tcx.def_kind(def_id), DefKind::Struct | DefKind::Enum)
        && matches!(
            cx.tcx.item_name(def_id).as_str(),
            "NodeInputOf" | "LayoutInputOf"
        )
        && path_eq(&definition_module_names(cx, def_id), &["node_input"])
}

fn is_layout_tree_node_input(cx: &LateContext<'_>, def_id: rustc_hir::def_id::DefId) -> bool {
    let trait_item_id = cx.tcx.trait_item_of(def_id).unwrap_or(def_id);
    cx.tcx.item_name(trait_item_id) == Symbol::intern("node_input")
        && cx
            .tcx
            .trait_of_assoc(trait_item_id)
            .is_some_and(|trait_id| {
                cx.tcx.item_name(trait_id) == Symbol::intern("LayoutTree")
                    && path_eq(&definition_module_names(cx, trait_id), &["tree"])
            })
}

fn is_catalog_scope(cx: &LateContext<'_>, hir_id: HirId) -> bool {
    is_audited_tree(cx, hir_id) || path_eq(&module_names(cx, hir_id), &["node_projection"])
}

fn is_audited_tree(cx: &LateContext<'_>, hir_id: HirId) -> bool {
    module_names(cx, hir_id)
        .first()
        .is_some_and(|name| AUDITED_TREES.contains(&name.as_str()))
}

fn is_projection_owner(cx: &LateContext<'_>, hir_id: HirId) -> bool {
    let names = module_names(cx, hir_id);
    path_eq(&names, &["node_projection"]) || INPUT_OWNERS.iter().any(|owner| path_eq(&names, owner))
}

fn module_names(cx: &LateContext<'_>, hir_id: HirId) -> Vec<Symbol> {
    definition_module_names(cx, hir_id.owner.def_id.to_def_id())
}

fn definition_module_names(
    cx: &LateContext<'_>,
    mut def_id: rustc_hir::def_id::DefId,
) -> Vec<Symbol> {
    let mut names = Vec::new();
    loop {
        if !def_id.is_crate_root()
            && cx.tcx.def_kind(def_id) == DefKind::Mod
            && let Some(name) = cx.tcx.opt_item_name(def_id)
        {
            names.push(name);
        }
        let Some(parent) = cx.tcx.opt_parent(def_id) else {
            break;
        };
        def_id = parent;
    }
    names.reverse();
    names
}

fn path_eq(actual: &[Symbol], expected: &[&str]) -> bool {
    actual
        .iter()
        .map(Symbol::as_str)
        .eq(expected.iter().copied())
}

fn is_test_only(cx: &LateContext<'_>, hir_id: HirId) -> bool {
    cx.tcx.sess.is_test_crate()
        || std::iter::once(hir_id)
            .chain(
                cx.tcx
                    .hir_parent_iter(hir_id)
                    .map(|(parent_id, _)| parent_id),
            )
            .any(|parent_id| {
                cx.tcx.hir_attrs(parent_id).iter().any(|attr| {
                    attr.has_name(sym::test)
                        || (attr.has_name(sym::cfg)
                            && attr.meta_item_list().is_some_and(|items| {
                                items.iter().any(|item| item.has_name(sym::test))
                            }))
                })
            })
}

fn is_within_type_alias(cx: &LateContext<'_>, hir_id: HirId) -> bool {
    cx.tcx.hir_parent_iter(hir_id).any(|(_, node)| {
        matches!(
            node,
            Node::Item(Item {
                kind: ItemKind::TyAlias(..),
                ..
            })
        )
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::process::Command;

    #[test]
    fn node_projection_boundary_ui() {
        build_test_library();
        install_test_library_name();
        dylint_testing::ui_test(env!("CARGO_PKG_NAME"), "ui");
        dylint_testing::ui::Test::src_base(env!("CARGO_PKG_NAME"), "ui-test-only")
            .rustc_flags(["--test"])
            .run();
    }

    fn build_test_library() {
        let manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
        let status = Command::new(std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into()))
            .args(["build", "--locked", "--offline", "--manifest-path"])
            .arg(&manifest_path)
            .status()
            .unwrap_or_else(|error| panic!("failed to build the Dylint test library: {error}"));
        assert!(status.success(), "failed to build the Dylint test library");
    }

    fn install_test_library_name() {
        let target_dir = PathBuf::from(
            std::env::var_os("CARGO_TARGET_DIR")
                .expect("catalog UI tests require the configured root target directory"),
        );
        let profile_dir = target_dir.join("debug");
        let source = profile_dir.join(format!(
            "{}{}{}",
            std::env::consts::DLL_PREFIX,
            env!("CARGO_CRATE_NAME"),
            std::env::consts::DLL_SUFFIX
        ));
        let toolchain = std::env::var("RUSTUP_TOOLCHAIN")
            .expect("catalog UI tests require the pinned rustup toolchain identity");
        let destination = profile_dir.join(format!(
            "{}{}@{}{}",
            std::env::consts::DLL_PREFIX,
            env!("CARGO_CRATE_NAME"),
            toolchain,
            std::env::consts::DLL_SUFFIX
        ));
        std::fs::copy(&source, &destination).unwrap_or_else(|error| {
            panic!(
                "failed to install Dylint test library {} as {}: {error}",
                source.display(),
                destination.display()
            )
        });
    }
}
