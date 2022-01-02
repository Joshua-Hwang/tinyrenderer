use cgmath::{InnerSpace, Matrix, Matrix4, Vector2, Vector3, Vector4};
use image::{GrayImage, Luma, Rgb, RgbImage};

use super::model;

pub const DEPTH: f32 = 255.0;
const EPSILON: f32 = 1e-2;

pub fn viewport(x: f32, y: f32, width: f32, height: f32) -> Matrix4<f32> {
    // translations to the centre of the desired rectangle
    // and scaling to the width and height
    Matrix4::<f32>::new(
        width / 2.0,
        0.0,
        0.0,
        0.0,
        0.0,
        height / 2.0,
        0.0,
        0.0,
        0.0,
        0.0,
        DEPTH / 2.0,
        0.0,
        x + width / 2.0,
        y + height / 2.0,
        DEPTH / 2.0,
        1.0,
    )
}

pub fn projection(coeff: f32) -> Matrix4<f32> {
    Matrix4::<f32>::new(
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, coeff, 1.0,
    )
    .transpose()
}

pub fn lookat(eye: Vector3<f32>, center: Vector3<f32>, up: Vector3<f32>) -> Matrix4<f32> {
    let z = (eye - center).normalize();
    let x = up.cross(z).normalize();
    let y = z.cross(x).normalize(); // can't use up since not necessarily orthogonal

    let minv = Matrix4::<f32>::from_cols(
        x.extend(0.0),
        y.extend(0.0),
        z.extend(0.0),
        Vector4::<f32>::new(0.0, 0.0, 0.0, 1.0),
    )
    .transpose();
    // tr translates our center to the center vector
    let tr = Matrix4::<f32>::from_cols(
        Vector4::<f32>::new(1.0, 0.0, 0.0, 0.0),
        Vector4::<f32>::new(0.0, 1.0, 0.0, 0.0),
        Vector4::<f32>::new(0.0, 0.0, 1.0, 0.0),
        -center.extend(-1.0), // negative * negative to have positive bottom right entry
    );

    minv * tr
}

// create interface (pretty sure that isn't possible in rust)
pub trait Shader {
    fn vertex(
        &mut self,
        model: &model::Model,
        iface: usize,
        nthvert: usize,
        mat: Matrix4<f32>,
    ) -> Vector4<f32>;
    // bar stands for barycentric coordinates
    fn fragment(&self, bar: Vector3<f32>, color: &mut Rgb<u8>) -> bool;
}

fn barycentric(pts: &[Vector2<f32>; 3], p: Vector2<f32>) -> Vector3<f32> {
    // Let a triangle be labeled ABC which are located at pts[0] pts[1] and pts[2]
    let x = Vector3::new(pts[2].x - pts[0].x, pts[1].x - pts[0].x, pts[0].x - p.x);
    let y = Vector3::new(pts[2].y - pts[0].y, pts[1].y - pts[0].y, pts[0].y - p.y);
    let u = x.cross(y);
    if u.z.abs() > EPSILON {
        Vector3::new(1.0 - (u.x + u.y) / u.z, u.y / u.z, u.x / u.z)
    } else {
        Vector3::new(-1.0, 1.0, 1.0)
    }
}

pub fn triangle<T: Shader>(
    pts: &[Vector4<f32>; 3], // TODO screen coords
    shader: &T,
    image: &mut RgbImage,
    zbuffer: &mut GrayImage,
) {
    let mut bboxmin: Vector2<i32> = Vector2::new(i32::MAX, i32::MAX);
    let mut bboxmax: Vector2<i32> = Vector2::new(-i32::MAX, -i32::MAX);
    for i in 0..3 {
        for j in 0..2 {
            if pts[i][j].is_sign_negative() {
                print!("Triangle outside bounds of canvas\n");
                return;
            }
            bboxmin[j] = bboxmin[j].min((pts[i][j] / pts[i].w) as i32);
            bboxmax[j] = bboxmax[j].max((pts[i][j] / pts[i].w) as i32);
        }
    }
    let pts_2d = pts.map(|pt| Vector2::new(pt.x / pt.w, pt.y / pt.w));
    for x in bboxmin.x..=bboxmax.x {
        for y in bboxmin.y..=bboxmax.y {
            let p: Vector2<f32> = Vector2::new(x as f32, y as f32);
            let c = barycentric(&pts_2d, p);

            let z = pts[0].z * c.x + pts[1].z * c.y + pts[2].z * c.z;
            let w = pts[0].w * c.x + pts[1].w * c.y + pts[2].w * c.z;

            let frag_depth = (z / w).clamp(0.0, 255.0) as u8;
            if c.x < 0.0
                || c.y < 0.0
                || c.z < 0.0
                || zbuffer.get_pixel(p.x as u32, p.y as u32)[0] >= frag_depth
            {
                continue;
            }
            //print!("{} {} {}\n", pts[0].z, pts[1].z, pts[2].z);

            let mut color: Rgb<u8> = Rgb([0, 0, 0]);
            let keep = shader.fragment(c, &mut color);
            if keep {
                zbuffer.put_pixel(p.x as u32, p.y as u32, Luma { 0: [frag_depth] });
                image.put_pixel(p.x as u32, p.y as u32, color);
            }
        }
    }
}
