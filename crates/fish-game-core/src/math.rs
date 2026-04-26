//! Deterministic math primitives.
//!
//! Core must never call `f32::sin`/`cos`/`sqrt` or `Vec3::normalize` directly —
//! those can select hardware-specific lowering (e.g. x87 vs SSE on some
//! toolchains) and diverge between native and wasm. Route everything through
//! `libm`, which is pure-Rust softfloat and bit-identical across targets.

use glam::{Vec2, Vec3};

#[inline]
pub fn sinf(x: f32) -> f32 {
    libm::sinf(x)
}

#[inline]
pub fn cosf(x: f32) -> f32 {
    libm::cosf(x)
}

#[inline]
pub fn sqrtf(x: f32) -> f32 {
    libm::sqrtf(x)
}

#[inline]
pub fn vec3_length(v: Vec3) -> f32 {
    sqrtf(v.x * v.x + v.y * v.y + v.z * v.z)
}

#[inline]
pub fn vec3_normalize_or_zero(v: Vec3) -> Vec3 {
    let len = vec3_length(v);
    if len > 0.0 {
        Vec3::new(v.x / len, v.y / len, v.z / len)
    } else {
        Vec3::ZERO
    }
}

#[inline]
pub fn vec2_length(v: Vec2) -> f32 {
    sqrtf(v.x * v.x + v.y * v.y)
}
