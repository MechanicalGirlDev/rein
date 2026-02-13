//! Contact data structures for collision response.

use std::collections::HashMap;

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

/// Cached contact data for warm-starting the solver.
#[derive(Debug, Clone, Copy)]
struct CachedContact {
    /// Contact position in world space (used for matching).
    position: Vec3,
    /// Accumulated normal impulse from previous frame.
    normal_impulse: f32,
    /// Accumulated tangent impulses from previous frame.
    tangent_impulse: [f32; 2],
}

/// Maximum distance squared for matching contacts across frames.
const CONTACT_MATCH_THRESHOLD_SQ: f32 = 0.02 * 0.02;

/// Cache of contact impulses for warm-starting the constraint solver.
///
/// Stores accumulated impulses from the previous frame keyed by entity pair.
/// On each new frame, current contacts are matched against cached contacts
/// by position proximity, and matching impulses are applied before the solver
/// iterates, greatly improving convergence.
#[derive(Debug, Default)]
pub struct ContactCache {
    cache: HashMap<(hecs::Entity, hecs::Entity), Vec<CachedContact>>,
}

impl ContactCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Apply cached impulses to current manifolds (warm start).
    pub fn warm_start(&self, manifolds: &mut [ContactManifold]) {
        for manifold in manifolds.iter_mut() {
            let key = Self::pair_key(manifold.entity_a, manifold.entity_b);
            if let Some(cached) = self.cache.get(&key) {
                for contact in &mut manifold.contacts {
                    // Find matching cached contact by position proximity
                    if let Some(cc) = cached.iter().min_by(|a, b| {
                        let da = (a.position - contact.position).length_squared();
                        let db = (b.position - contact.position).length_squared();
                        da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
                    }) {
                        let dist_sq = (cc.position - contact.position).length_squared();
                        if dist_sq < CONTACT_MATCH_THRESHOLD_SQ {
                            contact.normal_impulse = cc.normal_impulse;
                            contact.tangent_impulse = cc.tangent_impulse;
                        }
                    }
                }
            }
        }
    }

    /// Update cache with current frame's solved contacts.
    pub fn update(&mut self, manifolds: &[ContactManifold]) {
        self.cache.clear();
        for manifold in manifolds {
            let key = Self::pair_key(manifold.entity_a, manifold.entity_b);
            let contacts: Vec<CachedContact> = manifold
                .contacts
                .iter()
                .map(|c| CachedContact {
                    position: c.position,
                    normal_impulse: c.normal_impulse,
                    tangent_impulse: c.tangent_impulse,
                })
                .collect();
            self.cache.insert(key, contacts);
        }
    }

    /// Canonical pair key (smaller entity first).
    fn pair_key(a: hecs::Entity, b: hecs::Entity) -> (hecs::Entity, hecs::Entity) {
        if a < b {
            (a, b)
        } else {
            (b, a)
        }
    }
}
