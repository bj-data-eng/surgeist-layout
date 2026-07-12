use crate::*;
use crate::{
    Available, ComputeInput, Dimension, Edges, Length, LengthPercentageOf, NodeInput, RequestedAxis,
};

fn invalid_numeric_affine_value() -> LengthPercentageOf {
    LengthPercentageOf::from_coefficients(f32::MAX, 1.0).expect("finite coefficients")
}

fn invalid_numeric_affine_input() -> ComputeInput {
    ComputeInput {
        run_mode: RunMode::PerformLayout,
        sizing_mode: SizingMode::InherentSize,
        axis: RequestedAxis::Both,
        known: Size::NONE,
        parent: Size::new(Some(f32::MAX), None),
        available: Size::new(Available::definite(100.0), Available::MAX_CONTENT),
    }
}

#[test]
fn completed_batch_exposes_read_only_layout_and_cache_entries() {
    let unrounded = NodeOutput {
        size: Size::new(10.25, 20.5),
        ..NodeOutput::default()
    };
    let final_layout = NodeOutput {
        size: Size::new(10.0, 21.0),
        ..NodeOutput::default()
    };
    let cache_output = ComputeOutput::from_sizes(Size::new(10.25, 20.5), Size::new(8.0, 16.0));

    let batch = CompletedLayoutBatch::from_entries(
        vec![LayoutOutputEntry::new(7, unrounded)],
        vec![LayoutOutputEntry::new(7, final_layout)],
        vec![LayoutCacheStoreEntry::new(7, cache_output)],
        vec![LayoutCacheClearEntry::new(11)],
    );

    assert_eq!(batch.unrounded_entries()[0].node(), 7);
    assert_eq!(batch.unrounded_entries()[0].output(), unrounded);
    assert_eq!(batch.final_entries()[0].node(), 7);
    assert_eq!(batch.final_entries()[0].output(), final_layout);
    assert_eq!(batch.cache_store_entries()[0].node(), 7);
    assert_eq!(batch.cache_store_entries()[0].output(), cache_output);
    assert_eq!(batch.cache_clear_entries()[0].node(), 11);
}

#[test]
fn layout_error_accessors_preserve_typed_site_operation_kind_and_source() {
    #[derive(Clone, Debug, Eq, PartialEq)]
    struct ProviderSource(&'static str);

    let error = LayoutError::new(
        LayoutErrorSite::ContainerSubject {
            container: 1,
            subject: 2,
        },
        LayoutOperation::LeafMeasurement,
        LayoutErrorKind::Measurement(ProviderSource("measure failed")),
    );

    assert_eq!(
        error.site(),
        LayoutErrorSite::ContainerSubject {
            container: 1,
            subject: 2,
        }
    );
    assert_eq!(error.operation(), LayoutOperation::LeafMeasurement);
    assert_eq!(
        error.kind(),
        &LayoutErrorKind::Measurement(ProviderSource("measure failed"))
    );

    let scalar_error = LayoutError::<u32, ProviderSource>::new(
        LayoutErrorSite::Node(3),
        LayoutOperation::RootLayout,
        LayoutErrorKind::InvalidInput(LayoutInvalidInput::RootAvailability {
            axis: Axis::Horizontal,
            error: NonNegativeFiniteScalarErrorOf::Negative { value: -2.0 },
        }),
    );

    assert_eq!(
        scalar_error.kind(),
        &LayoutErrorKind::InvalidInput(LayoutInvalidInput::RootAvailability {
            axis: Axis::Horizontal,
            error: NonNegativeFiniteScalarErrorOf::Negative { value: -2.0 },
        })
    );
}

#[test]
fn leaf_affine_width_resolves_against_parent_basis() {
    let width = LengthPercentageOf::from_coefficients(10.0, 0.5).expect("finite coefficients");
    let style = NodeInput {
        size: Size::new(Dimension::value(width), Dimension::AUTO),
        ..NodeInput::default()
    };
    let input = ComputeInput {
        run_mode: RunMode::PerformLayout,
        sizing_mode: SizingMode::InherentSize,
        axis: RequestedAxis::Both,
        known: Size::NONE,
        parent: Size::new(Some(100.0), None),
        available: Size::new(Available::definite(100.0), Available::MAX_CONTENT),
    };

    let output = compute_leaf(input, &style, |_input| Ok::<_, ()>(Size::new(12.0, 8.0))).unwrap();

    assert_eq!(output.size.width, 60.0);
}

#[test]
fn public_leaf_affine_px_width_needs_no_resolver() {
    let width = LengthPercentageOf::px(10.0).expect("finite px");
    let style = NodeInput {
        size: Size::new(Dimension::value(width), Dimension::AUTO),
        ..NodeInput::default()
    };
    let input = ComputeInput {
        run_mode: RunMode::PerformLayout,
        sizing_mode: SizingMode::InherentSize,
        axis: RequestedAxis::Both,
        known: Size::NONE,
        parent: Size::new(Some(100.0), None),
        available: Size::new(Available::definite(100.0), Available::MAX_CONTENT),
    };

    let output = compute_leaf(input, &style, |_input| Ok::<_, ()>(Size::new(12.0, 8.0))).unwrap();

    assert_eq!(output.size.width, 10.0);
}

#[test]
fn leaf_invalid_numeric_affine_width_falls_back_to_measured_size() {
    let style = NodeInput {
        size: Size::new(
            Dimension::value(invalid_numeric_affine_value()),
            Dimension::AUTO,
        ),
        ..NodeInput::default()
    };

    let output = compute_leaf(invalid_numeric_affine_input(), &style, |measure_input| {
        let known = measure_input.known_content_size();
        let available = measure_input
            .available_content_size()
            .map(MeasurementAvailable::into_available);
        assert_eq!(known, Size::NONE);
        assert_eq!(
            available,
            Size::new(Available::definite(100.0), Available::MAX_CONTENT)
        );
        Ok::<_, ()>(Size::new(12.0, 8.0))
    })
    .unwrap();

    assert_eq!(output.size, Size::new(12.0, 8.0));
    assert_eq!(output.content_size, Size::new(12.0, 8.0));
}

#[test]
fn leaf_invalid_numeric_affine_padding_falls_back_to_zero() {
    let style = NodeInput {
        padding: Edges::all(Length::value(invalid_numeric_affine_value())),
        ..NodeInput::default()
    };

    let output = compute_leaf(invalid_numeric_affine_input(), &style, |measure_input| {
        let known = measure_input.known_content_size();
        let available = measure_input
            .available_content_size()
            .map(MeasurementAvailable::into_available);
        assert_eq!(known, Size::NONE);
        assert_eq!(
            available,
            Size::new(Available::definite(100.0), Available::MAX_CONTENT)
        );
        Ok::<_, ()>(Size::new(12.0, 8.0))
    })
    .unwrap();

    assert_eq!(output.size, Size::new(12.0, 8.0));
    assert_eq!(output.content_size, Size::new(12.0, 8.0));
}
