use std::collections::HashMap;

use bevy::camera::primitives::{Aabb, MeshAabb};
use bevy::math::{IVec2, UVec2, Vec2, Vec3};
use bevy::mesh::{Indices, Mesh, PrimitiveTopology, VertexAttributeValues};
use bevy::prelude::Vec3Swizzles;

#[derive(Clone, Debug)]
pub(crate) struct SurfaceTriangle {
    pub positions: [Vec3; 3],
    pub normals: [Vec3; 3],
    pub uvs: [Vec2; 3],
    pub area: f32,
}

impl SurfaceTriangle {
    pub fn sample_point(&self, barycentric: Vec3) -> Vec3 {
        self.positions[0] * barycentric.x
            + self.positions[1] * barycentric.y
            + self.positions[2] * barycentric.z
    }

    pub fn sample_normal(&self, barycentric: Vec3) -> Vec3 {
        (self.normals[0] * barycentric.x
            + self.normals[1] * barycentric.y
            + self.normals[2] * barycentric.z)
            .normalize_or_zero()
    }

    pub fn sample_uv(&self, barycentric: Vec3) -> Vec2 {
        self.uvs[0] * barycentric.x + self.uvs[1] * barycentric.y + self.uvs[2] * barycentric.z
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ChunkLayout {
    pub min: Vec2,
    pub max: Vec2,
    pub chunk_size: Vec2,
    pub dims: UVec2,
}

impl ChunkLayout {
    pub fn from_aabb(aabb: &Aabb, chunk_size: Vec2) -> Self {
        let min = aabb.min().xz();
        let max = aabb.max().xz();
        let span = (max - min).max(Vec2::splat(0.001));
        let chunk_size = chunk_size.max(Vec2::splat(0.001));
        let dims = UVec2::new(
            (span.x / chunk_size.x).ceil().max(1.0) as u32,
            (span.y / chunk_size.y).ceil().max(1.0) as u32,
        );
        Self {
            min,
            max,
            chunk_size,
            dims,
        }
    }

    pub fn coord_of_local_point(&self, point: Vec2) -> IVec2 {
        let relative = point - self.min;
        IVec2::new(
            (relative.x / self.chunk_size.x).floor() as i32,
            (relative.y / self.chunk_size.y).floor() as i32,
        )
    }

    pub fn contains_coord(&self, coord: IVec2) -> bool {
        coord.x >= 0 && coord.y >= 0 && coord.x < self.dims.x as i32 && coord.y < self.dims.y as i32
    }

    pub fn bounds_for_coord(&self, coord: IVec2) -> (Vec2, Vec2) {
        let min = self.min
            + Vec2::new(
                coord.x as f32 * self.chunk_size.x,
                coord.y as f32 * self.chunk_size.y,
            );
        (min, (min + self.chunk_size).min(self.max))
    }

    pub fn center_for_coord(&self, coord: IVec2) -> Vec3 {
        let (min, max) = self.bounds_for_coord(coord);
        let center = (min + max) * 0.5;
        Vec3::new(center.x, 0.0, center.y)
    }

    pub fn uv_of_local_point(&self, point: Vec2) -> Vec2 {
        let span = (self.max - self.min).max(Vec2::splat(0.001));
        ((point - self.min) / span).clamp(Vec2::ZERO, Vec2::ONE)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SurfaceBake {
    pub layout: ChunkLayout,
    pub triangles: Vec<SurfaceTriangle>,
    pub chunk_triangles: HashMap<IVec2, Vec<usize>>,
}

impl SurfaceBake {
    pub fn triangle_indices(&self, coord: IVec2) -> &[usize] {
        self.chunk_triangles
            .get(&coord)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
}

pub(crate) fn bake_mesh_surface(mesh: &Mesh, chunk_size: Vec2) -> Option<SurfaceBake> {
    if mesh.primitive_topology() != PrimitiveTopology::TriangleList {
        return None;
    }

    let positions = match mesh.attribute(Mesh::ATTRIBUTE_POSITION)? {
        VertexAttributeValues::Float32x3(values) => values,
        _ => return None,
    };
    let normals = match mesh.attribute(Mesh::ATTRIBUTE_NORMAL) {
        Some(VertexAttributeValues::Float32x3(values)) => Some(values),
        _ => None,
    };
    let uvs = match mesh.attribute(Mesh::ATTRIBUTE_UV_0) {
        Some(VertexAttributeValues::Float32x2(values)) => Some(values),
        _ => None,
    };

    let aabb = mesh.compute_aabb().unwrap_or_else(|| {
        let mut min = Vec3::splat(f32::INFINITY);
        let mut max = Vec3::splat(f32::NEG_INFINITY);
        for position in positions {
            let p = Vec3::from_array(*position);
            min = min.min(p);
            max = max.max(p);
        }
        Aabb::from_min_max(min, max)
    });
    let layout = ChunkLayout::from_aabb(&aabb, chunk_size);

    let indices: Vec<u32> = match mesh.indices() {
        Some(Indices::U16(values)) => values.iter().copied().map(u32::from).collect(),
        Some(Indices::U32(values)) => values.clone(),
        None => (0..positions.len() as u32).collect(),
    };

    let mut triangles = Vec::new();
    let mut chunk_triangles: HashMap<IVec2, Vec<usize>> = HashMap::new();

    for tri in indices.chunks_exact(3) {
        let [a, b, c] = [tri[0] as usize, tri[1] as usize, tri[2] as usize];
        let positions = [
            Vec3::from_array(positions[a]),
            Vec3::from_array(positions[b]),
            Vec3::from_array(positions[c]),
        ];
        let face_normal = (positions[1] - positions[0])
            .cross(positions[2] - positions[0])
            .normalize_or_zero();
        let area = 0.5
            * (positions[1] - positions[0])
                .cross(positions[2] - positions[0])
                .length();
        if area <= 0.000_001 {
            continue;
        }

        let triangle = SurfaceTriangle {
            positions,
            normals: [
                normals
                    .map(|values| Vec3::from_array(values[a]).normalize_or_zero())
                    .unwrap_or(face_normal),
                normals
                    .map(|values| Vec3::from_array(values[b]).normalize_or_zero())
                    .unwrap_or(face_normal),
                normals
                    .map(|values| Vec3::from_array(values[c]).normalize_or_zero())
                    .unwrap_or(face_normal),
            ],
            uvs: [
                uvs.map(|values| Vec2::from_array(values[a]))
                    .unwrap_or(Vec2::ZERO),
                uvs.map(|values| Vec2::from_array(values[b]))
                    .unwrap_or(Vec2::ZERO),
                uvs.map(|values| Vec2::from_array(values[c]))
                    .unwrap_or(Vec2::ZERO),
            ],
            area,
        };
        let bounds_min = positions[0]
            .xz()
            .min(positions[1].xz())
            .min(positions[2].xz());
        let bounds_max = positions[0]
            .xz()
            .max(positions[1].xz())
            .max(positions[2].xz());
        let tri_index = triangles.len();
        let min_coord = layout.coord_of_local_point(bounds_min);
        let max_coord = layout.coord_of_local_point(bounds_max);

        for y in min_coord.y..=max_coord.y {
            for x in min_coord.x..=max_coord.x {
                let coord = IVec2::new(x, y);
                if layout.contains_coord(coord) {
                    chunk_triangles.entry(coord).or_default().push(tri_index);
                }
            }
        }
        triangles.push(triangle);
    }

    Some(SurfaceBake {
        layout,
        triangles,
        chunk_triangles,
    })
}

#[cfg(test)]
#[path = "surface_tests.rs"]
mod tests;
