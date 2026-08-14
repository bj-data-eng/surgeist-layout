use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
};

use proptest::prelude::*;

use crate::flex::FlexAxes;
use crate::geometry::PhysicalProgression;
use crate::test_support::layout_tree::{
    OracleMeasurementOf, OracleTree, OracleTreeOf, PublicLayoutTreeOf,
};
use crate::test_support::scroll_geometry::{
    assert_geometry_error as fri06_mr02_geometry_error_assert, assert_scroll_padding_inputs_exact,
    geometry_error_input as fri06_mr02_geometry_error_input,
    geometry_error_largest_finite as fri06_mr02_geometry_error_largest_finite,
    scroll_padding_inputs,
};
use crate::*;

#[path = "flex_tests/fixtures_tests.rs"]
mod fixtures;

#[path = "flex_tests/alignment_baselines_tests.rs"]
mod alignment_baselines;
#[path = "flex_tests/intrinsic_absolute_scroll_tests.rs"]
mod intrinsic_absolute_scroll;
#[path = "flex_tests/items_tests.rs"]
mod items;
#[path = "flex_tests/lines_distribution_tests.rs"]
mod lines_distribution;
