use glam::Vec2;

/// Axis-aligned bounding box overlap test. Centers + half-sizes.
#[inline]
pub fn aabb_overlap(a_center: Vec2, a_half: Vec2, b_center: Vec2, b_half: Vec2) -> bool {
    let dx = (a_center.x - b_center.x).abs();
    let dy = (a_center.y - b_center.y).abs();
    dx < (a_half.x + b_half.x) && dy < (a_half.y + b_half.y)
}
