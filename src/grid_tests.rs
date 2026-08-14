use std::collections::{HashMap, HashSet};

use super::lanes::*;
use super::tracks::*;
use super::*;
use crate::geometry::{
    LogicalAxis, LogicalPointOf, LogicalSizeOf, PhysicalAxis, PhysicalProgression,
};
use crate::test_support::{
    self as lts,
    layout_tree::{OracleMeasurement, OracleTree, OracleTreeOf, PublicLayoutTreeOf},
};
use crate::*;

use lts::oracle::grid::{
    AlignmentSafety, AutoPlacer, ContributionSize, DefiniteTracks, Flow,
    GridArea as OracleGridArea, GridTrack, ItemContributionFacts, LinePlacement, Track,
    TrackAlignment, TrackSizingSlice, align_tracks_report,
};

#[macro_use]
#[path = "grid_tests/fixtures_tests.rs"]
mod fixtures;

#[path = "grid_tests/browser_controls_tests.rs"]
mod browser_controls;
#[path = "grid_tests/child_baseline_tests.rs"]
mod child_baseline;
#[path = "grid_tests/lanes_subgrid_tests.rs"]
mod lanes_subgrid;
#[path = "grid_tests/oracle_comparison_tests.rs"]
mod oracle_comparison;
#[path = "grid_tests/scroll_composition_tests.rs"]
mod scroll_composition;
#[path = "grid_tests/topology_placement_tests.rs"]
mod topology_placement;
#[path = "grid_tests/tracks_intrinsic_tests.rs"]
mod tracks_intrinsic;
