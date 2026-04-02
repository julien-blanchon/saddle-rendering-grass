use super::*;

#[test]
fn later_lod_band_overlaps_before_previous_band_ends() {
    let lod = GrassLodConfig::default();
    let bands = resolve_lod_bands(&lod);

    assert_eq!(bands[0].visibility_range.start_margin, 0.0..0.0);
    assert_eq!(
        bands[1].visibility_range.start_margin.end,
        lod.bands[0].max_distance
    );
    assert_eq!(
        bands[2].visibility_range.start_margin.end,
        lod.bands[1].max_distance
    );
    assert!(
        bands[1].visibility_range.start_margin.start < lod.bands[0].max_distance,
        "band 1 should fade in before band 0 fully ends"
    );
    assert!(
        bands[2].visibility_range.start_margin.start < lod.bands[1].max_distance,
        "band 2 should fade in before band 1 fully ends"
    );
}

#[test]
fn end_margin_extends_by_fade_distance() {
    let lod = GrassLodConfig::default();
    let bands = resolve_lod_bands(&lod);

    for band in bands {
        assert_eq!(
            band.visibility_range.end_margin.end,
            band.band.max_distance + band.band.fade_distance
        );
    }
}
