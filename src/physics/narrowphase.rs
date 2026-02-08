//! Narrowphase collision detection: GJK, EPA, and specialized tests.

use glam::Vec3;

use crate::ecs::components::physics::ColliderShape;
use crate::ecs::components::transform::GlobalTransform;

use super::contact::ContactInfo;

/// A simplex used by the GJK algorithm (up to 4 vertices in 3D).
#[derive(Debug, Clone)]
pub struct Simplex {
    pub points: Vec<Vec3>,
}

impl Simplex {
    fn new() -> Self {
        Self {
            points: Vec::with_capacity(4),
        }
    }

    fn push(&mut self, point: Vec3) {
        self.points.push(point);
    }
}

/// Minkowski difference support function.
fn minkowski_support(
    shape_a: &ColliderShape,
    transform_a: &GlobalTransform,
    shape_b: &ColliderShape,
    transform_b: &GlobalTransform,
    direction: Vec3,
) -> Vec3 {
    let a = shape_a.support(direction, transform_a);
    let b = shape_b.support(-direction, transform_b);
    a - b
}

/// GJK intersection test. Returns Some(simplex) if shapes intersect, None otherwise.
pub fn gjk_intersection(
    shape_a: &ColliderShape,
    transform_a: &GlobalTransform,
    shape_b: &ColliderShape,
    transform_b: &GlobalTransform,
) -> Option<Simplex> {
    let mut direction = Vec3::X; // Initial arbitrary direction

    let mut simplex = Simplex::new();

    // First support point
    let first = minkowski_support(shape_a, transform_a, shape_b, transform_b, direction);
    simplex.push(first);
    direction = -first;

    if direction.length_squared() < 1e-10 {
        // Shapes overlap at exactly one point
        return Some(simplex);
    }

    // Second support point
    let second = minkowski_support(shape_a, transform_a, shape_b, transform_b, direction);
    if second.dot(direction) < 0.0 {
        return None;
    }
    simplex.push(second);
    direction = triple_cross_product(second - first, -first, second - first);
    if direction.length_squared() < 1e-10 {
        direction = (second - first).any_orthonormal_vector();
    }

    for _ in 0..64 {
        let new_point = minkowski_support(shape_a, transform_a, shape_b, transform_b, direction);
        if new_point.dot(direction) < 0.0 {
            return None;
        }
        simplex.push(new_point);

        if do_simplex(&mut simplex, &mut direction) {
            return Some(simplex);
        }

        if direction.length_squared() < 1e-10 {
            return Some(simplex);
        }
    }

    None
}

/// Triple cross product: (a x b) x c
fn triple_cross_product(a: Vec3, b: Vec3, c: Vec3) -> Vec3 {
    a.cross(b).cross(c)
}

/// Process the simplex and update the search direction.
/// Returns true if the origin is contained in the simplex.
fn do_simplex(simplex: &mut Simplex, direction: &mut Vec3) -> bool {
    match simplex.points.len() {
        2 => do_simplex_line(simplex, direction),
        3 => do_simplex_triangle(simplex, direction),
        4 => do_simplex_tetrahedron(simplex, direction),
        _ => false,
    }
}

fn do_simplex_line(simplex: &mut Simplex, direction: &mut Vec3) -> bool {
    let a = simplex.points[1]; // Most recently added
    let b = simplex.points[0];
    let ab = b - a;
    let ao = -a;

    if ab.dot(ao) > 0.0 {
        *direction = triple_cross_product(ab, ao, ab);
    } else {
        simplex.points = vec![a];
        *direction = ao;
    }
    false
}

fn do_simplex_triangle(simplex: &mut Simplex, direction: &mut Vec3) -> bool {
    let a = simplex.points[2]; // Most recently added
    let b = simplex.points[1];
    let c = simplex.points[0];
    let ab = b - a;
    let ac = c - a;
    let ao = -a;
    let abc = ab.cross(ac);

    if abc.cross(ac).dot(ao) > 0.0 {
        if ac.dot(ao) > 0.0 {
            simplex.points = vec![c, a];
            *direction = triple_cross_product(ac, ao, ac);
        } else {
            simplex.points = vec![b, a];
            return do_simplex_line(simplex, direction);
        }
    } else if ab.cross(abc).dot(ao) > 0.0 {
        simplex.points = vec![b, a];
        return do_simplex_line(simplex, direction);
    } else {
        // Origin is above or below the triangle
        if abc.dot(ao) > 0.0 {
            *direction = abc;
        } else {
            simplex.points = vec![b, c, a];
            *direction = -abc;
        }
    }
    false
}

fn do_simplex_tetrahedron(simplex: &mut Simplex, direction: &mut Vec3) -> bool {
    let a = simplex.points[3]; // Most recently added
    let b = simplex.points[2];
    let c = simplex.points[1];
    let d = simplex.points[0];
    let ab = b - a;
    let ac = c - a;
    let ad = d - a;
    let ao = -a;

    let abc = ab.cross(ac);
    let acd = ac.cross(ad);
    let adb = ad.cross(ab);

    if abc.dot(ao) > 0.0 {
        simplex.points = vec![c, b, a];
        *direction = abc;
        return do_simplex_triangle(simplex, direction);
    }
    if acd.dot(ao) > 0.0 {
        simplex.points = vec![d, c, a];
        *direction = acd;
        return do_simplex_triangle(simplex, direction);
    }
    if adb.dot(ao) > 0.0 {
        simplex.points = vec![b, d, a];
        *direction = adb;
        return do_simplex_triangle(simplex, direction);
    }

    // Origin is inside the tetrahedron
    true
}

/// EPA (Expanding Polytope Algorithm) to compute penetration depth and contact normal.
pub fn epa_penetration(
    simplex: &Simplex,
    shape_a: &ColliderShape,
    transform_a: &GlobalTransform,
    shape_b: &ColliderShape,
    transform_b: &GlobalTransform,
) -> Option<ContactInfo> {
    const EPA_TOLERANCE: f32 = 1e-4;
    const MAX_EPA_ITERATIONS: usize = 64;

    // Build initial polytope from the GJK simplex
    let mut polytope = simplex.points.clone();
    if polytope.len() < 4 {
        // Need a tetrahedron for EPA - try to build one
        return epa_fallback(shape_a, transform_a, shape_b, transform_b, &polytope);
    }

    // Faces as indices into the polytope (counter-clockwise winding, normals pointing outward)
    let mut faces: Vec<[usize; 3]> = vec![[0, 1, 2], [0, 3, 1], [0, 2, 3], [1, 3, 2]];

    for _ in 0..MAX_EPA_ITERATIONS {
        // Find the face closest to the origin
        let mut min_dist = f32::MAX;
        let mut min_face = 0;
        let mut min_normal = Vec3::ZERO;

        for (i, face) in faces.iter().enumerate() {
            let a = polytope[face[0]];
            let b = polytope[face[1]];
            let c = polytope[face[2]];
            let normal = (b - a).cross(c - a);
            let len = normal.length();
            if len < 1e-10 {
                continue;
            }
            let normal = normal / len;
            let dist = normal.dot(a);

            // Ensure normal points away from origin
            let (normal, dist) = if dist < 0.0 {
                (-normal, -dist)
            } else {
                (normal, dist)
            };

            if dist < min_dist {
                min_dist = dist;
                min_face = i;
                min_normal = normal;
            }
        }

        if min_normal == Vec3::ZERO {
            return None;
        }

        // Get a new support point along the closest face's normal
        let new_point = minkowski_support(shape_a, transform_a, shape_b, transform_b, min_normal);
        let new_dist = new_point.dot(min_normal);

        if new_dist - min_dist < EPA_TOLERANCE {
            // Converged - compute contact point
            let face = faces[min_face];
            let a = polytope[face[0]];
            let point_on_minkowski =
                closest_point_on_triangle(a, polytope[face[1]], polytope[face[2]]);

            // Approximate the contact point as the support of shape_a in the normal direction
            let contact_point = shape_a.support(min_normal, transform_a);

            return Some(ContactInfo {
                normal: min_normal,
                penetration: min_dist,
                point: contact_point - min_normal * (point_on_minkowski.dot(min_normal) * 0.5),
            });
        }

        // Expand the polytope
        let new_idx = polytope.len();
        polytope.push(new_point);

        // Remove faces that can see the new point
        let mut edges: Vec<[usize; 2]> = Vec::new();
        let mut i = 0;
        while i < faces.len() {
            let face = faces[i];
            let a = polytope[face[0]];
            let b = polytope[face[1]];
            let c = polytope[face[2]];
            let normal = (b - a).cross(c - a);
            let len = normal.length();
            if len < 1e-10 {
                faces.swap_remove(i);
                continue;
            }
            let normal = normal / len;

            if normal.dot(new_point - a) > 0.0 {
                // This face can see the new point - remove it and add its edges
                add_edge(&mut edges, face[0], face[1]);
                add_edge(&mut edges, face[1], face[2]);
                add_edge(&mut edges, face[2], face[0]);
                faces.swap_remove(i);
            } else {
                i += 1;
            }
        }

        // Create new faces from the edges to the new point
        for edge in &edges {
            faces.push([edge[0], edge[1], new_idx]);
        }

        if faces.is_empty() {
            return None;
        }
    }

    None
}

/// Add an edge to the edge list, removing duplicates (shared edges).
fn add_edge(edges: &mut Vec<[usize; 2]>, a: usize, b: usize) {
    // Check if the reverse edge already exists
    if let Some(pos) = edges.iter().position(|e| e[0] == b && e[1] == a) {
        edges.swap_remove(pos);
    } else {
        edges.push([a, b]);
    }
}

/// Find the closest point on a triangle to the origin.
fn closest_point_on_triangle(a: Vec3, b: Vec3, c: Vec3) -> Vec3 {
    let ab = b - a;
    let ac = c - a;
    let ao = -a;

    let d1 = ab.dot(ao);
    let d2 = ac.dot(ao);
    if d1 <= 0.0 && d2 <= 0.0 {
        return a;
    }

    let bo = -b;
    let d3 = ab.dot(bo);
    let d4 = ac.dot(bo);
    if d3 >= 0.0 && d4 <= d3 {
        return b;
    }

    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        return a + ab * v;
    }

    let co = -c;
    let d5 = ab.dot(co);
    let d6 = ac.dot(co);
    if d6 >= 0.0 && d5 <= d6 {
        return c;
    }

    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        return a + ac * w;
    }

    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return b + (c - b) * w;
    }

    let denom = 1.0 / (va + vb + vc);
    let v = vb * denom;
    let w = vc * denom;
    a + ab * v + ac * w
}

/// Fallback for when GJK doesn't produce a full tetrahedron.
fn epa_fallback(
    shape_a: &ColliderShape,
    transform_a: &GlobalTransform,
    shape_b: &ColliderShape,
    transform_b: &GlobalTransform,
    _simplex: &[Vec3],
) -> Option<ContactInfo> {
    // Try specialized tests first
    sphere_sphere(shape_a, transform_a, shape_b, transform_b)
}

/// Specialized sphere-sphere intersection test.
pub fn sphere_sphere(
    shape_a: &ColliderShape,
    transform_a: &GlobalTransform,
    shape_b: &ColliderShape,
    transform_b: &GlobalTransform,
) -> Option<ContactInfo> {
    let (radius_a, radius_b) = match (shape_a, shape_b) {
        (ColliderShape::Sphere { radius: ra }, ColliderShape::Sphere { radius: rb }) => (*ra, *rb),
        _ => return None,
    };

    let center_a = transform_a.0.transform_point3(Vec3::ZERO);
    let center_b = transform_b.0.transform_point3(Vec3::ZERO);

    // Account for scale
    let scale_a = transform_a.0.x_axis.truncate().length();
    let scale_b = transform_b.0.x_axis.truncate().length();
    let world_radius_a = radius_a * scale_a;
    let world_radius_b = radius_b * scale_b;

    let diff = center_b - center_a;
    let dist_sq = diff.length_squared();
    let min_dist = world_radius_a + world_radius_b;

    if dist_sq >= min_dist * min_dist {
        return None;
    }

    let dist = dist_sq.sqrt();
    let normal = if dist > 1e-6 { diff / dist } else { Vec3::Y };

    let penetration = min_dist - dist;
    let point = center_a + normal * (world_radius_a - penetration * 0.5);

    Some(ContactInfo {
        normal,
        penetration,
        point,
    })
}

/// SAT (Separating Axis Theorem) test for box-box collision.
pub fn sat_box_box(
    half_a: Vec3,
    transform_a: glam::Mat4,
    half_b: Vec3,
    transform_b: glam::Mat4,
) -> Option<ContactInfo> {
    let center_a = transform_a.transform_point3(Vec3::ZERO);
    let center_b = transform_b.transform_point3(Vec3::ZERO);

    // Extract axes (rotation columns)
    let axes_a = [
        transform_a.x_axis.truncate().normalize_or_zero(),
        transform_a.y_axis.truncate().normalize_or_zero(),
        transform_a.z_axis.truncate().normalize_or_zero(),
    ];
    let axes_b = [
        transform_b.x_axis.truncate().normalize_or_zero(),
        transform_b.y_axis.truncate().normalize_or_zero(),
        transform_b.z_axis.truncate().normalize_or_zero(),
    ];

    let half_a_arr = [half_a.x, half_a.y, half_a.z];
    let half_b_arr = [half_b.x, half_b.y, half_b.z];

    let t = center_b - center_a;

    let mut min_overlap = f32::MAX;
    let mut best_axis = Vec3::ZERO;

    // Test 15 axes: 3 from A, 3 from B, 9 cross products
    // A's face normals
    for i in 0..3 {
        let axis = axes_a[i];
        if let Some(overlap) = sat_test_axis(axis, &axes_a, &half_a_arr, &axes_b, &half_b_arr, t) {
            if overlap < min_overlap {
                min_overlap = overlap;
                best_axis = axis;
            }
        } else {
            return None;
        }
    }

    // B's face normals
    for i in 0..3 {
        let axis = axes_b[i];
        if let Some(overlap) = sat_test_axis(axis, &axes_a, &half_a_arr, &axes_b, &half_b_arr, t) {
            if overlap < min_overlap {
                min_overlap = overlap;
                best_axis = axis;
            }
        } else {
            return None;
        }
    }

    // Edge-edge cross products
    for i in 0..3 {
        for j in 0..3 {
            let axis = axes_a[i].cross(axes_b[j]);
            let len = axis.length();
            if len < 1e-6 {
                continue; // Parallel edges
            }
            let axis = axis / len;
            if let Some(overlap) =
                sat_test_axis(axis, &axes_a, &half_a_arr, &axes_b, &half_b_arr, t)
            {
                if overlap < min_overlap {
                    min_overlap = overlap;
                    best_axis = axis;
                }
            } else {
                return None;
            }
        }
    }

    // Ensure normal points from A to B
    if best_axis.dot(t) < 0.0 {
        best_axis = -best_axis;
    }

    let point = center_a + best_axis * (half_a.dot(best_axis.abs()) - min_overlap * 0.5);

    Some(ContactInfo {
        normal: best_axis,
        penetration: min_overlap,
        point,
    })
}

/// Test a single SAT axis. Returns Some(overlap) if overlapping, None if separating.
fn sat_test_axis(
    axis: Vec3,
    axes_a: &[Vec3; 3],
    half_a: &[f32; 3],
    axes_b: &[Vec3; 3],
    half_b: &[f32; 3],
    t: Vec3,
) -> Option<f32> {
    let mut proj_a = 0.0f32;
    for i in 0..3 {
        proj_a += half_a[i] * axes_a[i].dot(axis).abs();
    }
    let mut proj_b = 0.0f32;
    for i in 0..3 {
        proj_b += half_b[i] * axes_b[i].dot(axis).abs();
    }

    let dist = t.dot(axis).abs();
    let overlap = proj_a + proj_b - dist;

    if overlap > 0.0 {
        Some(overlap)
    } else {
        None
    }
}

/// Detect collision between two shapes, dispatching to specialized tests where possible.
pub fn detect_collision(
    shape_a: &ColliderShape,
    transform_a: &GlobalTransform,
    shape_b: &ColliderShape,
    transform_b: &GlobalTransform,
) -> Option<ContactInfo> {
    // Try specialized tests first
    match (shape_a, shape_b) {
        (ColliderShape::Sphere { .. }, ColliderShape::Sphere { .. }) => {
            sphere_sphere(shape_a, transform_a, shape_b, transform_b)
        }
        (
            ColliderShape::Box {
                half_extents: half_a,
            },
            ColliderShape::Box {
                half_extents: half_b,
            },
        ) => sat_box_box(*half_a, transform_a.0, *half_b, transform_b.0),
        _ => {
            // General GJK + EPA
            let simplex = gjk_intersection(shape_a, transform_a, shape_b, transform_b)?;
            epa_penetration(&simplex, shape_a, transform_a, shape_b, transform_b)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Mat4;

    #[test]
    fn test_sphere_sphere_intersection() {
        let shape_a = ColliderShape::Sphere { radius: 1.0 };
        let shape_b = ColliderShape::Sphere { radius: 1.0 };
        let transform_a = GlobalTransform(Mat4::IDENTITY);
        let transform_b = GlobalTransform(Mat4::from_translation(Vec3::new(1.5, 0.0, 0.0)));

        let result = sphere_sphere(&shape_a, &transform_a, &shape_b, &transform_b);
        assert!(result.is_some());

        let info = result.unwrap();
        let eps = 1e-4;
        assert!((info.normal - Vec3::X).length() < eps);
        assert!((info.penetration - 0.5).abs() < eps);
    }

    #[test]
    fn test_sphere_sphere_no_intersection() {
        let shape_a = ColliderShape::Sphere { radius: 1.0 };
        let shape_b = ColliderShape::Sphere { radius: 1.0 };
        let transform_a = GlobalTransform(Mat4::IDENTITY);
        let transform_b = GlobalTransform(Mat4::from_translation(Vec3::new(3.0, 0.0, 0.0)));

        let result = sphere_sphere(&shape_a, &transform_a, &shape_b, &transform_b);
        assert!(result.is_none());
    }

    #[test]
    fn test_gjk_spheres_intersecting() {
        let shape_a = ColliderShape::Sphere { radius: 1.0 };
        let shape_b = ColliderShape::Sphere { radius: 1.0 };
        let transform_a = GlobalTransform(Mat4::IDENTITY);
        let transform_b = GlobalTransform(Mat4::from_translation(Vec3::new(1.0, 0.0, 0.0)));

        let result = gjk_intersection(&shape_a, &transform_a, &shape_b, &transform_b);
        assert!(result.is_some());
    }

    #[test]
    fn test_gjk_spheres_not_intersecting() {
        let shape_a = ColliderShape::Sphere { radius: 1.0 };
        let shape_b = ColliderShape::Sphere { radius: 1.0 };
        let transform_a = GlobalTransform(Mat4::IDENTITY);
        let transform_b = GlobalTransform(Mat4::from_translation(Vec3::new(5.0, 0.0, 0.0)));

        let result = gjk_intersection(&shape_a, &transform_a, &shape_b, &transform_b);
        assert!(result.is_none());
    }

    #[test]
    fn test_sat_box_box_intersection() {
        let half_a = Vec3::splat(1.0);
        let half_b = Vec3::splat(1.0);
        let transform_a = Mat4::IDENTITY;
        let transform_b = Mat4::from_translation(Vec3::new(1.5, 0.0, 0.0));

        let result = sat_box_box(half_a, transform_a, half_b, transform_b);
        assert!(result.is_some());
        let info = result.unwrap();
        assert!(info.penetration > 0.0);
    }

    #[test]
    fn test_sat_box_box_no_intersection() {
        let half_a = Vec3::splat(1.0);
        let half_b = Vec3::splat(1.0);
        let transform_a = Mat4::IDENTITY;
        let transform_b = Mat4::from_translation(Vec3::new(3.0, 0.0, 0.0));

        let result = sat_box_box(half_a, transform_a, half_b, transform_b);
        assert!(result.is_none());
    }

    #[test]
    fn test_detect_collision_dispatch() {
        // Sphere-sphere should use specialized path
        let shape_a = ColliderShape::Sphere { radius: 1.0 };
        let shape_b = ColliderShape::Sphere { radius: 1.0 };
        let transform_a = GlobalTransform(Mat4::IDENTITY);
        let transform_b = GlobalTransform(Mat4::from_translation(Vec3::new(1.0, 0.0, 0.0)));

        let result = detect_collision(&shape_a, &transform_a, &shape_b, &transform_b);
        assert!(result.is_some());

        // Box-box should use SAT
        let shape_a = ColliderShape::Box {
            half_extents: Vec3::splat(1.0),
        };
        let shape_b = ColliderShape::Box {
            half_extents: Vec3::splat(1.0),
        };
        let result = detect_collision(&shape_a, &transform_a, &shape_b, &transform_b);
        assert!(result.is_some());
    }
}
