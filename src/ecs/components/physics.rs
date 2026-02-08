//! Physics components for ECS entities.

use glam::Vec3;

/// Rigid body type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RigidBodyType {
    /// Affected by forces and collisions.
    Dynamic,
    /// Immovable.
    Static,
    /// Position controlled by user, but affects dynamic bodies.
    Kinematic,
}

/// Rigid body component.
#[derive(Debug, Clone)]
pub struct RigidBody {
    pub body_type: RigidBodyType,
    pub mass: f32,
    /// Inertia tensor stored as column-major 3x3 matrix.
    pub inertia_tensor: [f32; 9],
    pub linear_velocity: Vec3,
    pub angular_velocity: Vec3,
    pub force_accumulator: Vec3,
    pub torque_accumulator: Vec3,
    /// Linear damping factor (default: 0.01).
    pub linear_damping: f32,
    /// Angular damping factor (default: 0.01).
    pub angular_damping: f32,
    /// Coefficient of restitution (0.0 - 1.0).
    pub restitution: f32,
    /// Friction coefficient (0.0 - 1.0).
    pub friction: f32,
    /// Gravity scale (default: 1.0).
    pub gravity_scale: f32,
}

impl RigidBody {
    /// Create a new dynamic rigid body with the given mass.
    pub fn new_dynamic(mass: f32) -> Self {
        // Default inertia tensor: identity * mass (unit sphere approximation)
        let i = mass;
        Self {
            body_type: RigidBodyType::Dynamic,
            mass,
            inertia_tensor: [i, 0.0, 0.0, 0.0, i, 0.0, 0.0, 0.0, i],
            linear_velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            force_accumulator: Vec3::ZERO,
            torque_accumulator: Vec3::ZERO,
            linear_damping: 0.01,
            angular_damping: 0.01,
            restitution: 0.3,
            friction: 0.5,
            gravity_scale: 1.0,
        }
    }

    /// Create a new static rigid body.
    pub fn new_static() -> Self {
        Self {
            body_type: RigidBodyType::Static,
            mass: 0.0,
            inertia_tensor: [0.0; 9],
            linear_velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            force_accumulator: Vec3::ZERO,
            torque_accumulator: Vec3::ZERO,
            linear_damping: 0.0,
            angular_damping: 0.0,
            restitution: 0.3,
            friction: 0.5,
            gravity_scale: 0.0,
        }
    }

    /// Create a new kinematic rigid body.
    pub fn new_kinematic() -> Self {
        Self {
            body_type: RigidBodyType::Kinematic,
            mass: 0.0,
            inertia_tensor: [0.0; 9],
            linear_velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            force_accumulator: Vec3::ZERO,
            torque_accumulator: Vec3::ZERO,
            linear_damping: 0.0,
            angular_damping: 0.0,
            restitution: 0.3,
            friction: 0.5,
            gravity_scale: 0.0,
        }
    }
}

/// Collider shape.
#[derive(Debug, Clone)]
pub enum ColliderShape {
    Sphere { radius: f32 },
    Box { half_extents: Vec3 },
    Capsule { radius: f32, half_height: f32 },
    Cylinder { radius: f32, half_height: f32 },
    ConvexHull { points: Vec<Vec3> },
}

/// Collision detection component.
#[derive(Debug, Clone)]
pub struct Collider {
    pub shape: ColliderShape,
    /// Offset from the entity's transform origin.
    pub offset: Vec3,
    /// If true, generates collision events but no physics response.
    pub is_sensor: bool,
}

impl Default for Collider {
    fn default() -> Self {
        Self {
            shape: ColliderShape::Sphere { radius: 0.5 },
            offset: Vec3::ZERO,
            is_sensor: false,
        }
    }
}
