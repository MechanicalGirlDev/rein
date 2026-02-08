//! Transform components for ECS entities.

use glam::{Mat4, Quat, Vec3};

/// Local-space transform. Stores position, rotation, and scale separately.
#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Transform {
    /// Create an identity transform.
    pub fn identity() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }

    /// Create a transform from a position.
    pub fn from_position(position: Vec3) -> Self {
        Self {
            position,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }

    /// Convert to a 4x4 matrix (translation * rotation * scale).
    pub fn to_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }

    /// Decompose a 4x4 matrix into a Transform.
    ///
    /// Note: This assumes the matrix represents a valid affine transform
    /// (no shear). Non-uniform scale with rotation may lose precision.
    pub fn from_matrix(mat: Mat4) -> Self {
        let (scale, rotation, position) = mat.to_scale_rotation_translation();
        Self {
            position,
            rotation,
            scale,
        }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}

/// World-space transform matrix. Updated every frame by TransformSystem.
#[derive(Debug, Clone, Copy)]
pub struct GlobalTransform(pub Mat4);

impl Default for GlobalTransform {
    fn default() -> Self {
        Self(Mat4::IDENTITY)
    }
}

/// Reference to a parent entity.
pub struct Parent(pub hecs::Entity);

/// List of child entities.
pub struct Children(pub Vec<hecs::Entity>);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity() {
        let t = Transform::identity();
        assert_eq!(t.position, Vec3::ZERO);
        assert_eq!(t.rotation, Quat::IDENTITY);
        assert_eq!(t.scale, Vec3::ONE);
        assert_eq!(t.to_matrix(), Mat4::IDENTITY);
    }

    #[test]
    fn test_from_position() {
        let pos = Vec3::new(1.0, 2.0, 3.0);
        let t = Transform::from_position(pos);
        assert_eq!(t.position, pos);
        assert_eq!(t.rotation, Quat::IDENTITY);
        assert_eq!(t.scale, Vec3::ONE);
    }

    #[test]
    fn test_to_matrix_roundtrip() {
        let original = Transform {
            position: Vec3::new(1.0, 2.0, 3.0),
            rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_4),
            scale: Vec3::new(2.0, 1.5, 0.5),
        };

        let mat = original.to_matrix();
        let recovered = Transform::from_matrix(mat);

        let eps = 1e-5;
        assert!((original.position - recovered.position).length() < eps);
        // Quaternion can be negated and still represent the same rotation
        let dot = original.rotation.dot(recovered.rotation).abs();
        assert!((dot - 1.0).abs() < eps);
        assert!((original.scale - recovered.scale).length() < eps);
    }

    #[test]
    fn test_default_is_identity() {
        let t = Transform::default();
        assert_eq!(t.to_matrix(), Mat4::IDENTITY);
    }

    #[test]
    fn test_global_transform_default() {
        let gt = GlobalTransform::default();
        assert_eq!(gt.0, Mat4::IDENTITY);
    }
}
