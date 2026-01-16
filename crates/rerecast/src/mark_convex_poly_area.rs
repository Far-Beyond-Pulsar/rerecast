use alloc::vec::Vec;
use glam::{IVec3, Vec2};

use crate::{Aabb2d, AreaType, CompactHeightfield};

impl CompactHeightfield {
    /// Sets the [`AreaType`] of the spans within the given convex volume.
    pub fn mark_convex_poly_area(&mut self, volume: &ConvexVolume) {
        // Early-out for empty polygon
        if volume.vertices.is_empty() {
            return;
        }

        // Compute the bounding box of the polygon
        let Some(aabb) = Aabb2d::from_verts(&volume.vertices) else {
            // The volume is empty
            return;
        };
        let aabb = aabb.extend_y(volume.min_y, volume.max_y);

        // Cache inverse for faster division
        let inverse_cell_size = 1.0 / self.cell_size;
        let inverse_cell_height = 1.0 / self.cell_height;

        // Compute the grid footprint of the polygon
        let mut min = aabb.min - self.aabb.min;
        min.x *= inverse_cell_size;
        min.y *= inverse_cell_height;
        min.z *= inverse_cell_size;
        let mut max = aabb.max - self.aabb.min;
        max.x *= inverse_cell_size;
        max.y *= inverse_cell_height;
        max.z *= inverse_cell_size;
        let mut min = IVec3::new(min.x as i32, min.y as i32, min.z as i32);
        let mut max = IVec3::new(max.x as i32, max.y as i32, max.z as i32);

        // Early-out if the polygon lies entirely outside the grid.
        if max.x < 0 || min.x >= self.width as i32 || max.z < 0 || min.z >= self.height as i32 {
            return;
        }

        // Clamp the polygon footprint to the grid
        min.x = min.x.max(0);
        max.x = max.x.min(self.width as i32 - 1);
        min.z = min.z.max(0);
        max.z = max.z.min(self.height as i32 - 1);

        // Optimized: Cache constant values and hoist point calculation out of innermost loop
        let width_i32 = self.width as i32;
        let area = volume.area;
        let min_y_i32 = min.y;
        let max_y_i32 = max.y;

        for z in min.z..=max.z {
            // Precompute Z component of point (constant for this row)
            let point_z = self.aabb.min.z + (z as f32 + 0.5) * self.cell_size;

            for x in min.x..=max.x {
                // Precompute point for this cell (constant for all spans in this cell)
                let point = Vec2::new(
                    self.aabb.min.x + (x as f32 + 0.5) * self.cell_size,
                    point_z,
                );

                // Check if point is in polygon once per cell, not per span
                if !point_in_poly_fast(&point, &volume.vertices) {
                    continue;
                }

                let cell_index = (x + z * width_i32) as usize;
                let cell = &self.cells[cell_index];
                let max_index = cell.index() as usize + cell.count() as usize;

                for i in cell.index() as usize..max_index {
                    let span = &self.spans[i];

                    // Skip if span is removed
                    if !self.areas[i].is_walkable() {
                        continue;
                    }

                    // Skip if y extents don't overlap
                    let span_y = span.y as i32;
                    if span_y < min_y_i32 || span_y > max_y_i32 {
                        continue;
                    }

                    self.areas[i] = area;
                }
            }
        }
    }
}

// Optimized point-in-polygon test with inlining
#[inline]
fn point_in_poly_fast(point: &Vec2, vertices: &[Vec2]) -> bool {
    let mut inside = false;
    let mut j = vertices.len() - 1;
    let px = point.x;
    let py = point.y;

    for i in 0..vertices.len() {
        let vi = &vertices[i];
        let vj = &vertices[j];
        let xi = vi.x;
        let yi = vi.y;
        let xj = vj.x;
        let yj = vj.y;

        // Optimized: Use bitwise XOR for != comparison and avoid division when possible
        if ((yi > py) != (yj > py))
            && (px < (xj - xi) * (py - yi) / (yj - yi) + xi)
        {
            inside = !inside;
        }
        j = i;
    }
    inside
}

/// A convex volume that marks an area within a [`CompactHeightfield`] as belonging to a specific [`AreaType`] through [`CompactHeightfield::mark_convex_poly_area`].
///
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct ConvexVolume {
    /// The vertices of the convex volume. In 3D, these represent the X and Z coordinates of the vertices.
    pub vertices: Vec<Vec2>,
    /// The lower Y coordinate of the convex volume.
    pub min_y: f32,
    /// The upper Y coordinate of the convex volume.
    pub max_y: f32,
    /// The area type of the convex volume.
    pub area: AreaType,
}
