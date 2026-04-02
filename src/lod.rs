use bevy::camera::visibility::VisibilityRange;

use crate::config::{GrassLodBand, GrassLodConfig};

#[derive(Clone)]
pub(crate) struct SelectedLodBand {
    pub index: usize,
    pub band: GrassLodBand,
    pub visibility_range: VisibilityRange,
}

pub(crate) fn visibility_range_for_band(bands: &[GrassLodBand], index: usize) -> VisibilityRange {
    let band = &bands[index];
    let start = if index == 0 {
        0.0
    } else {
        bands[index - 1].max_distance
    };
    let start_fade = if index == 0 {
        0.0
    } else {
        (start - band.fade_distance.max(0.0)).max(0.0)
    };

    VisibilityRange {
        start_margin: start_fade..start,
        end_margin: band.max_distance..(band.max_distance + band.fade_distance.max(0.0)),
        use_aabb: false,
    }
}

pub(crate) fn resolve_lod_bands(config: &GrassLodConfig) -> [SelectedLodBand; 3] {
    [
        SelectedLodBand {
            index: 0,
            band: config.bands[0].clone(),
            visibility_range: visibility_range_for_band(&config.bands, 0),
        },
        SelectedLodBand {
            index: 1,
            band: config.bands[1].clone(),
            visibility_range: visibility_range_for_band(&config.bands, 1),
        },
        SelectedLodBand {
            index: 2,
            band: config.bands[2].clone(),
            visibility_range: visibility_range_for_band(&config.bands, 2),
        },
    ]
}

#[cfg(test)]
#[path = "lod_tests.rs"]
mod tests;
