//! Contact data structures for collision response.

use glam::Vec3;

/// Information about a single contact between two shapes.
#[derive(Debug, Clone, Copy)]
pub struct ContactInfo {
    /// Contact normal (from shape A to shape B).
    pub normal: Vec3,
    /// Penetration depth.
    pub penetration: f32,
    /// Contact point in world space.
    pub point: Vec3,
}

/// A single contact point with accumulated impulse data.
#[derive(Debug, Clone, Copy)]
pub struct ContactPoint {
    /// Contact position in world space.
    pub position: Vec3,
    /// Penetration depth.
    pub penetration: f32,
    /// Accumulated normal impulse.
    pub normal_impulse: f32,
    /// Accumulated tangent impulses (two friction directions).
    pub tangent_impulse: [f32; 2],
}

/// A collection of contact points between two entities.
#[derive(Debug, Clone)]
pub struct ContactManifold {
    pub entity_a: hecs::Entity,
    pub entity_b: hecs::Entity,
    /// Contact normal (from A to B).
    pub normal: Vec3,
    pub contacts: Vec<ContactPoint>,
}
