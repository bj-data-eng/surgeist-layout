use std::cell::Cell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Mutex;

use crate::block::{FloatExclusions, FloatLedgerSide, resolve_logical_in_flow_margin};
use crate::test_support::layout_tree::OracleTree;
use crate::test_support::scroll_geometry::{
    assert_geometry_error as fri06_mr02_geometry_error_assert, assert_scroll_padding_inputs_exact,
    geometry_error_input as fri06_mr02_geometry_error_input,
    geometry_error_largest_finite as fri06_mr02_geometry_error_largest_finite,
    scroll_padding_inputs,
};
use crate::*;

#[path = "block_tests/fixtures_tests.rs"]
mod fixtures;

#[path = "block_tests/absolute_tests.rs"]
mod absolute;
#[path = "block_tests/floats_bfcs_tests.rs"]
mod floats_bfcs;
#[path = "block_tests/in_flow_margins_tests.rs"]
mod in_flow_margins;
#[path = "block_tests/inline_runs_tests.rs"]
mod inline_runs;
#[path = "block_tests/sizing_scroll_tests.rs"]
mod sizing_scroll;
