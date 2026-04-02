use bevy::mesh::Mesh;
use bevy::prelude::*;

use super::*;

#[test]
fn planar_mesh_bakes_into_chunks() {
    let mesh = Mesh::from(Plane3d::default().mesh().size(8.0, 8.0).subdivisions(2));
    let bake = bake_mesh_surface(&mesh, Vec2::new(2.0, 2.0)).expect("plane should bake");

    assert!(!bake.triangles.is_empty());
    assert!(bake.layout.dims.x >= 1);
    assert!(bake.layout.dims.y >= 1);
}

#[test]
fn chunk_layout_bounds_round_trip() {
    let mesh = Mesh::from(Plane3d::default().mesh().size(4.0, 4.0));
    let bake = bake_mesh_surface(&mesh, Vec2::new(2.0, 2.0)).expect("plane should bake");
    let center = bake.layout.center_for_coord(IVec2::new(0, 0));
    assert!(center.length() <= 2.0);
}
