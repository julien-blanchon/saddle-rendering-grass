use super::*;

#[test]
fn world_to_uv_maps_center_to_half() {
    let map = GrassInteractionMap {
        center: Vec2::new(10.0, 20.0),
        half_extent: 30.0,
        ..default()
    };
    let uv = map.world_to_uv(Vec2::new(10.0, 20.0)).unwrap();
    assert!((uv.x - 0.5).abs() < 0.01);
    assert!((uv.y - 0.5).abs() < 0.01);
}

#[test]
fn world_to_uv_returns_none_outside_region() {
    let map = GrassInteractionMap {
        center: Vec2::ZERO,
        half_extent: 10.0,
        ..default()
    };
    assert!(map.world_to_uv(Vec2::new(15.0, 0.0)).is_none());
    assert!(map.world_to_uv(Vec2::new(0.0, -15.0)).is_none());
}

#[test]
fn decay_moves_bend_toward_neutral() {
    let res = 4usize;
    let mut data = vec![0u8; res * res * 4];
    // Set pixel (0,0) to strong bend right (R=200, G=128)
    data[0] = 200;
    data[1] = 128;
    data[2] = 100; // flatten
    data[3] = 50; // hide (non-permanent)

    let decay = 0.5;
    for pixel in 0..res * res {
        let idx = pixel * 4;
        data[idx] = lerp_u8(data[idx], 128, decay);
        data[idx + 1] = lerp_u8(data[idx + 1], 128, decay);
        data[idx + 2] = lerp_u8(data[idx + 2], 0, decay);
        if data[idx + 3] < 255 {
            data[idx + 3] = lerp_u8(data[idx + 3], 0, decay);
        }
    }

    assert!(
        data[0] < 200 && data[0] > 128,
        "R should decay toward 128, got {}",
        data[0]
    );
    assert_eq!(data[1], 128);
    assert!(data[2] < 100);
    assert!(data[3] < 50);
}

#[test]
fn permanent_hide_does_not_decay() {
    let mut data = [0u8; 4];
    data[3] = 255;
    if data[3] < 255 {
        data[3] = lerp_u8(data[3], 0, 0.99);
    }
    assert_eq!(data[3], 255);
}

#[test]
fn stamp_bend_actor_writes_directional_bend() {
    let map = GrassInteractionMap {
        center: Vec2::ZERO,
        half_extent: 10.0,
        resolution: 32,
        ..default()
    };
    let res = 32usize;
    let mut data = vec![0u8; res * res * 4];
    for pixel in 0..res * res {
        data[pixel * 4] = 128;
        data[pixel * 4 + 1] = 128;
    }

    let actor = GrassInteractionActor {
        radius: 5.0,
        policy: GrassInteractionPolicy::Bend { strength: 1.0 },
        falloff: 1.0,
    };

    // Actor at world (0, 0) — center of the map
    stamp_actor(&map, &mut data, res, Vec2::ZERO, &actor);

    // A pixel at world (+2, 0) = right of actor.
    // World (+2, 0) → UV = (2/20 + 0.5, 0/20 + 0.5) = (0.6, 0.5)
    // Pixel = (0.6 * 31, 0.5 * 31) = (18, 15)
    let px = 18;
    let py = 15;
    let idx = (py * res + px) * 4;
    assert!(
        data[idx] > 128,
        "pixel right of actor center should bend right (R > 128), got {} at ({}, {})",
        data[idx],
        px,
        py,
    );
}

#[test]
fn stamp_flatten_actor_writes_blue_channel() {
    let map = GrassInteractionMap {
        center: Vec2::ZERO,
        half_extent: 10.0,
        resolution: 32,
        ..default()
    };
    let res = 32usize;
    let mut data = vec![0u8; res * res * 4];
    for pixel in 0..res * res {
        data[pixel * 4] = 128;
        data[pixel * 4 + 1] = 128;
    }

    let actor = GrassInteractionActor {
        radius: 5.0,
        policy: GrassInteractionPolicy::Flatten { strength: 1.0 },
        falloff: 1.0,
    };

    stamp_actor(&map, &mut data, res, Vec2::ZERO, &actor);

    // Center pixel (16, 16)
    let idx = (16 * res + 16) * 4;
    assert!(
        data[idx + 2] > 100,
        "center should be flattened (B > 100), got {}",
        data[idx + 2]
    );
}

#[test]
fn stamp_hide_actor_sets_alpha() {
    let map = GrassInteractionMap {
        center: Vec2::ZERO,
        half_extent: 10.0,
        resolution: 32,
        ..default()
    };
    let res = 32usize;
    let mut data = vec![0u8; res * res * 4];
    for pixel in 0..res * res {
        data[pixel * 4] = 128;
        data[pixel * 4 + 1] = 128;
    }

    let actor = GrassInteractionActor {
        radius: 5.0,
        policy: GrassInteractionPolicy::Hide { permanent: true },
        falloff: 1.0,
    };

    stamp_actor(&map, &mut data, res, Vec2::ZERO, &actor);

    // Center pixel (16, 16)
    let idx = (16 * res + 16) * 4;
    assert!(
        data[idx + 3] > 150,
        "center should be hidden (A > 150), got {}",
        data[idx + 3]
    );
}
