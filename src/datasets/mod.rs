//! Built-in datasets.
//!
//! Datasets are embedded with `include_str!` so they work identically on
//! desktop and wasm — no file system access required at runtime.

pub mod iris;

use crate::geometry::{Point2, Point3};

/// A labelled 2D dataset with its display name and the points it produced.
pub struct Dataset2D {
    pub name: &'static str,
    pub points: Vec<Point2>,
}

/// A labelled 3D dataset with its display name and the points it produced.
pub struct Dataset3D {
    pub name: &'static str,
    pub points: Vec<Point3>,
}

/// Parse a simple CSV with optional header into rows of numeric features.
/// Each row may have a trailing non-numeric label column (e.g. species name)
/// which is dropped. The header is detected by checking whether the first
/// column of the first non-empty line parses as a float.
fn parse_csv_features(csv: &str) -> Vec<Vec<f32>> {
    let mut rows = Vec::new();
    let mut seen_first = false;
    for line in csv.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split(',').collect();
        if !seen_first {
            seen_first = true;
            // If the very first column doesn't parse as a number, it's a
            // header row — skip it.
            if cols.first().map(|c| c.trim().parse::<f32>().is_err()).unwrap_or(false) {
                continue;
            }
        }
        let mut row = Vec::with_capacity(cols.len());
        for c in &cols {
            if let Ok(v) = c.trim().parse::<f32>() {
                row.push(v);
            }
            // Non-numeric columns are silently dropped (e.g. species name).
        }
        if !row.is_empty() {
            rows.push(row);
        }
    }
    rows
}

/// Iris dataset projected to 2D using sepal length (x) and petal length (y).
/// Scaled into a 1000×700 world.
pub fn iris_2d() -> Dataset2D {
    let rows = parse_csv_features(iris::CSV);
    // Iris columns: sepal_length, sepal_width, petal_length, petal_width
    // Use sepal_length (col 0) for x and petal_length (col 2) for y;
    // these two give the clearest visual separation.
    let pts: Vec<Point2> = rows
        .iter()
        .filter(|r| r.len() >= 3)
        .map(|r| {
            // sepal_length: 4.3..7.9 -> map to 50..950
            let x = (r[0] - 4.3) / (7.9 - 4.3) * 900.0 + 50.0;
            // petal_length: 1.0..6.9 -> map to 50..650
            let y = (r[2] - 1.0) / (6.9 - 1.0) * 600.0 + 50.0;
            Point2::new(x, y)
        })
        .collect();
    Dataset2D { name: "iris (2D projection)", points: pts }
}

/// Iris dataset projected to 3D via PCA on all 4 features.
///
/// Standardizes each feature (zero mean, unit std-dev), computes the
/// 4×4 covariance matrix, finds the top 3 eigenvectors via power
/// iteration with deflation, and projects each sample onto them.
/// The resulting 3D cloud is scaled into a centered cube of half-side
/// `half_extent`.
pub fn iris_3d(half_extent: f32) -> Dataset3D {
    let rows = parse_csv_features(iris::CSV);
    let features: Vec<[f32; 4]> = rows
        .iter()
        .filter(|r| r.len() >= 4)
        .map(|r| [r[0], r[1], r[2], r[3]])
        .collect();

    let pts = pca_project_3d(&features, half_extent * 0.9);
    Dataset3D { name: "iris (PCA 3D)", points: pts }
}

/// Standardize a (n_samples × n_features) matrix in place: each column
/// gets zero mean and unit standard deviation. Columns with zero variance
/// are left alone (divide-by-zero protection).
fn standardize_columns(data: &mut [[f32; 4]]) {
    if data.is_empty() {
        return;
    }
    let n = data.len() as f32;
    for col in 0..4 {
        let mean: f32 = data.iter().map(|r| r[col]).sum::<f32>() / n;
        let var: f32 =
            data.iter().map(|r| (r[col] - mean).powi(2)).sum::<f32>() / (n - 1.0).max(1.0);
        let std = var.sqrt().max(1e-9);
        for row in data.iter_mut() {
            row[col] = (row[col] - mean) / std;
        }
    }
}

/// 4×4 covariance matrix of a standardized data matrix.
#[allow(clippy::needless_range_loop)] // index-based access is clearest for 4×4 matrix math
fn covariance_4(data: &[[f32; 4]]) -> [[f32; 4]; 4] {
    let n = data.len().max(1) as f32;
    let mut cov = [[0.0f32; 4]; 4];
    for row in data {
        for i in 0..4 {
            for j in 0..4 {
                cov[i][j] += row[i] * row[j];
            }
        }
    }
    let denom = (n - 1.0).max(1.0);
    for i in 0..4 {
        for j in 0..4 {
            cov[i][j] /= denom;
        }
    }
    cov
}

/// Find the dominant eigenvector of a symmetric 4×4 matrix via power
/// iteration. Returns `(eigenvalue, eigenvector)`. The eigenvector is
/// unit-length.
#[allow(clippy::needless_range_loop)] // index-based access is clearest for 4×4 matrix math
fn dominant_eigen_4(m: &[[f32; 4]; 4]) -> (f32, [f32; 4]) {
    // Deterministic starting guess so results are reproducible.
    let mut v = [1.0f32, 1.0, 1.0, 1.0];
    normalize_4(&mut v);

    let mut eigenvalue = 0.0;
    for _ in 0..200 {
        // w = m · v
        let mut w = [0.0f32; 4];
        for i in 0..4 {
            for j in 0..4 {
                w[i] += m[i][j] * v[j];
            }
        }
        // Rayleigh quotient = vᵀ · w (since v is unit-length).
        let new_eigenvalue: f32 = (0..4).map(|i| v[i] * w[i]).sum();
        let norm = (w[0] * w[0] + w[1] * w[1] + w[2] * w[2] + w[3] * w[3]).sqrt();
        if norm < 1e-9 {
            break;
        }
        for i in 0..4 {
            v[i] = w[i] / norm;
        }
        if (new_eigenvalue - eigenvalue).abs() < 1e-7 {
            eigenvalue = new_eigenvalue;
            break;
        }
        eigenvalue = new_eigenvalue;
    }
    (eigenvalue, v)
}

fn normalize_4(v: &mut [f32; 4]) {
    let n = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2] + v[3] * v[3]).sqrt();
    if n > 1e-9 {
        for x in v.iter_mut() {
            *x /= n;
        }
    }
}

/// Subtract the rank-1 contribution of eigenvalue λ and eigenvector v
/// from a symmetric matrix m. After deflation, `m'` has the same
/// remaining eigenstructure but with `v` removed. This lets us call
/// `dominant_eigen_4` again to get the next-largest eigenvector.
fn deflate_4(m: &mut [[f32; 4]; 4], lambda: f32, v: &[f32; 4]) {
    for i in 0..4 {
        for j in 0..4 {
            m[i][j] -= lambda * v[i] * v[j];
        }
    }
}

/// Project a 4D dataset to 3D using PCA, then scale into a centered
/// cube of half-side `half_extent`.
fn pca_project_3d(features: &[[f32; 4]], half_extent: f32) -> Vec<Point3> {
    if features.is_empty() {
        return Vec::new();
    }
    let mut data = features.to_vec();
    standardize_columns(&mut data);
    let mut cov = covariance_4(&data);

    // Top 3 eigenvectors via power iteration + deflation.
    let (lam1, v1) = dominant_eigen_4(&cov);
    deflate_4(&mut cov, lam1, &v1);
    let (lam2, v2) = dominant_eigen_4(&cov);
    deflate_4(&mut cov, lam2, &v2);
    let (_lam3, v3) = dominant_eigen_4(&cov);

    // Project each sample onto the three eigenvectors.
    let projected: Vec<[f32; 3]> = data
        .iter()
        .map(|row| {
            let mut out = [0.0f32; 3];
            for j in 0..4 {
                out[0] += row[j] * v1[j];
                out[1] += row[j] * v2[j];
                out[2] += row[j] * v3[j];
            }
            out
        })
        .collect();

    // Find per-axis ranges, then scale into the cube.
    let mut min_a = [f32::INFINITY; 3];
    let mut max_a = [f32::NEG_INFINITY; 3];
    for p in &projected {
        for i in 0..3 {
            if p[i] < min_a[i] {
                min_a[i] = p[i];
            }
            if p[i] > max_a[i] {
                max_a[i] = p[i];
            }
        }
    }
    projected
        .iter()
        .map(|p| {
            let scaled: [f32; 3] = std::array::from_fn(|i| {
                let range = (max_a[i] - min_a[i]).max(1e-9);
                ((p[i] - min_a[i]) / range) * 2.0 * half_extent - half_extent
            });
            Point3::new(scaled[0], scaled[1], scaled[2])
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iris_csv_parses_to_150_rows() {
        let rows = parse_csv_features(iris::CSV);
        assert_eq!(rows.len(), 150, "iris should have 150 samples");
    }

    #[test]
    fn iris_2d_points_in_world_bounds() {
        let ds = iris_2d();
        assert_eq!(ds.points.len(), 150);
        for p in &ds.points {
            assert!((0.0..=1000.0).contains(&p.x), "x out of bounds: {}", p.x);
            assert!((0.0..=700.0).contains(&p.y), "y out of bounds: {}", p.y);
        }
    }

    #[test]
    fn iris_3d_points_in_cube() {
        let ds = iris_3d(5.0);
        assert_eq!(ds.points.len(), 150);
        for p in &ds.points {
            assert!((-5.0..=5.0).contains(&p.x), "x out: {}", p.x);
            assert!((-5.0..=5.0).contains(&p.y), "y out: {}", p.y);
            assert!((-5.0..=5.0).contains(&p.z), "z out: {}", p.z);
        }
    }

    #[test]
    fn iris_dataset_name_is_set() {
        let ds = iris_2d();
        assert!(ds.name.contains("iris"));
        let ds3 = iris_3d(5.0);
        assert!(ds3.name.contains("iris"));
    }

    #[test]
    fn standardize_makes_zero_mean_unit_std() {
        let mut data = vec![[1.0, 10.0, 0.0, 5.0], [3.0, 20.0, 0.0, 7.0], [5.0, 30.0, 0.0, 9.0]];
        standardize_columns(&mut data);
        // Column means
        for col in 0..4 {
            let mean: f32 = data.iter().map(|r| r[col]).sum::<f32>() / 3.0;
            assert!(mean.abs() < 1e-5, "col {col} mean {mean}");
        }
        // Column 0 had variance, so should now be unit-std. Column 2 was
        // constant, so should still be all zeros (no division by zero).
        let std0: f32 =
            (data.iter().map(|r| r[0].powi(2)).sum::<f32>() / 2.0).sqrt();
        assert!((std0 - 1.0).abs() < 1e-4, "col0 std {std0}");
        for r in &data {
            assert!(r[2].abs() < 1e-5, "col2 should be all zeros");
        }
    }

    #[test]
    fn dominant_eigen_finds_known_eigenvector() {
        // Diagonal matrix: eigenvalues are diagonal entries, eigenvectors
        // are the standard basis. Largest eigenvalue is 9 with vector e1.
        let m = [
            [9.0, 0.0, 0.0, 0.0],
            [0.0, 4.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 0.5],
        ];
        let (lam, v) = dominant_eigen_4(&m);
        assert!((lam - 9.0).abs() < 1e-4, "got eigenvalue {lam}");
        // Eigenvector should be aligned with e0 (sign may flip).
        assert!(v[0].abs() > 0.99, "v[0] = {}", v[0]);
        assert!(v[1].abs() < 0.01);
        assert!(v[2].abs() < 0.01);
        assert!(v[3].abs() < 0.01);
    }

    #[test]
    fn deflation_removes_dominant_component() {
        let mut m = [
            [9.0, 0.0, 0.0, 0.0],
            [0.0, 4.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 0.5],
        ];
        let (lam1, v1) = dominant_eigen_4(&m);
        deflate_4(&mut m, lam1, &v1);
        // After deflation, the next-largest eigenvalue should be 4.
        let (lam2, _v2) = dominant_eigen_4(&m);
        assert!((lam2 - 4.0).abs() < 1e-3, "got {lam2}");
    }

    #[test]
    fn iris_pca_uses_more_variance_than_3_feature_slice() {
        // Sanity: PCA projection should spread points across the cube
        // more uniformly than the old 3-feature slice. Concretely, the
        // standard deviation along each principal axis should be non-trivial.
        let ds = iris_3d(5.0);
        let n = ds.points.len() as f32;
        let mean_x: f32 = ds.points.iter().map(|p| p.x).sum::<f32>() / n;
        let mean_y: f32 = ds.points.iter().map(|p| p.y).sum::<f32>() / n;
        let mean_z: f32 = ds.points.iter().map(|p| p.z).sum::<f32>() / n;
        let std_x: f32 = (ds.points.iter().map(|p| (p.x - mean_x).powi(2)).sum::<f32>() / n).sqrt();
        let std_y: f32 = (ds.points.iter().map(|p| (p.y - mean_y).powi(2)).sum::<f32>() / n).sqrt();
        let std_z: f32 = (ds.points.iter().map(|p| (p.z - mean_z).powi(2)).sum::<f32>() / n).sqrt();
        // Every axis should have at least 0.5 std (cube half-side is 5).
        assert!(std_x > 0.5, "PC1 std too small: {std_x}");
        assert!(std_y > 0.5, "PC2 std too small: {std_y}");
        assert!(std_z > 0.3, "PC3 std too small: {std_z}");
    }

    #[test]
    fn csv_parser_skips_alphabetic_header() {
        let csv = "a,b\n1.0,2.0\n3.0,4.0";
        let rows = parse_csv_features(csv);
        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn csv_parser_skips_empty_lines() {
        let csv = "1.0,2.0\n\n3.0,4.0\n";
        let rows = parse_csv_features(csv);
        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn csv_parser_drops_non_numeric_label_columns() {
        let csv = "5.1,3.5,1.4,0.2,Iris-setosa\n4.9,3.0,1.4,0.2,Iris-setosa";
        let rows = parse_csv_features(csv);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].len(), 4);
    }
}
