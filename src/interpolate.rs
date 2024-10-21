use ndarray::{Array1, Array2, ArrayView3};
use std::cmp::{max, min};

pub fn trilinear_interpolation(
    volgrid: (&Array1<f32>, &Array1<f32>, &Array1<f32>), // X, Y, Z grids
    data: ArrayView3<f32>,                               // 3D data array
    coords: &Array2<f32>,                                // Target coordinates (n, 3)
) -> Array1<f32> {
    let (X, Y, Z) = volgrid;
    let nx = data.shape()[0] as i32;
    let ny = data.shape()[1] as i32;
    let nz = data.shape()[2] as i32;

    let x_min = X[0];
    let x_max = X[X.len() - 1];
    let y_min = Y[0];
    let y_max = Y[Y.len() - 1];
    let z_min = Z[0];
    let z_max = Z[Z.len() - 1];

    // Vector to store interpolated results
    let mut result = Vec::with_capacity(coords.shape()[0]);

    // Iterate over each row of the coords matrix
    for coord in coords.outer_iter() {
        let x_idx = (coord[0] - x_min) / (x_max - x_min) * (nx as f32 - 1.0);
        let y_idx = (coord[1] - y_min) / (y_max - y_min) * (ny as f32 - 1.0);
        let z_idx = (coord[2] - z_min) / (z_max - z_min) * (nz as f32 - 1.0);

        // Integer and fractional parts
        let x0 = x_idx.floor() as i32;
        let y0 = y_idx.floor() as i32;
        let z0 = z_idx.floor() as i32;

        let x1 = min(max(x0 + 1, 0), nx - 1);
        let y1 = min(max(y0 + 1, 0), ny - 1);
        let z1 = min(max(z0 + 1, 0), nz - 1);

        let xd = x_idx - x0 as f32;
        let yd = y_idx - y0 as f32;
        let zd = z_idx - z0 as f32;

        // Interpolating the eight corners of the cube
        let c000 = data[[x0 as usize, y0 as usize, z0 as usize]];
        let c100 = data[[x1 as usize, y0 as usize, z0 as usize]];
        let c010 = data[[x0 as usize, y1 as usize, z0 as usize]];
        let c110 = data[[x1 as usize, y1 as usize, z0 as usize]];
        let c001 = data[[x0 as usize, y0 as usize, z1 as usize]];
        let c101 = data[[x1 as usize, y0 as usize, z1 as usize]];
        let c011 = data[[x0 as usize, y1 as usize, z1 as usize]];
        let c111 = data[[x1 as usize, y1 as usize, z1 as usize]];

        // Interpolate along x-axis
        let c00 = c000 * (1.0 - xd) + c100 * xd;
        let c10 = c010 * (1.0 - xd) + c110 * xd;
        let c01 = c001 * (1.0 - xd) + c101 * xd;
        let c11 = c011 * (1.0 - xd) + c111 * xd;

        // Interpolate along y-axis
        let c0 = c00 * (1.0 - yd) + c10 * yd;
        let c1 = c01 * (1.0 - yd) + c11 * yd;

        // Interpolate along z-axis
        let c = c0 * (1.0 - zd) + c1 * zd;

        result.push(c);
    }

    Array1::from_vec(result)
}