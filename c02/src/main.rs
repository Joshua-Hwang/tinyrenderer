use image::{Rgb, ImageBuffer, RgbImage, imageops};
use cgmath::{Vector3, Vector2, dot};

mod model;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 800;
const LIGHT_DIR: Vector3<f32> = Vector3{x: 0.0, y: 0.0, z: -1.0};

fn barycentric(pts: &[Vector2<i32>; 3], p: Vector2<i32>) -> Vector3<f32> {
    // Let a triangle be labeled ABC
    let x = Vector3::new(pts[2].x - pts[0].x, pts[1].x - pts[0].x, pts[0].x - p.x);
    let y = Vector3::new(pts[2].y - pts[0].y, pts[1].y - pts[0].y, pts[0].y - p.y);
    let u = x.cross(y);
    if u.z.abs() == 0 { Vector3::new(-1.0, 1.0, 1.0) } else { Vector3::new(1.0 - ((u.x + u.y) as f32)/(u.z as f32), (u.y as f32)/(u.z as f32), (u.x as f32)/(u.z as f32)) }
}

fn triangle(pts: &[Vector2<i32>; 3], image: &mut RgbImage, color: Rgb<u8>) {
    let mut bboxmin = Vector2::new((image.width() - 1) as i32, (image.height() - 1) as i32);
    let mut bboxmax = Vector2::new(0, 0);
    let clamp = Vector2::new((image.width() - 1) as i32, (image.height() - 1) as i32);
    for i in 0..3 {
        for j in 0..2 {
            bboxmin[j] = bboxmin[j].clamp(0, pts[i][j]);
            bboxmax[j] = bboxmax[j].max(pts[i][j]).min(clamp[j]);
        }
    }
    for x in bboxmin.x..=bboxmax.x {
        for y in bboxmin.y..=bboxmax.y {
            let p: Vector2<i32> = Vector2::new(x, y);
            let bc_screen = barycentric(&pts, p);
            if bc_screen.x.is_sign_positive() && bc_screen.y.is_sign_positive() && bc_screen.z.is_sign_positive() {
                image.put_pixel(x.try_into().unwrap(), y.try_into().unwrap(), color);
            } 
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let model = model::file_to_model(if args.len() == 2 { &args[1] } else { "obj/african_head.obj" }).unwrap();

    let mut image: RgbImage = ImageBuffer::new(WIDTH, HEIGHT);

    let verts = model.get_verts();
    for face in model.get_faces() {
        let mut screen_coords: [Vector2<i32>; 3] = [Vector2{x: 0, y: 0}; 3];
        let mut world_coords: [Vector3<f32>; 3] = [Vector3{x: 0.0, y: 0.0, z: 0.0}; 3];
        for j in 0..3usize {
            let v = verts[face[j]];
            screen_coords[j] = Vector2::new(((v.x + 1.0)*(WIDTH as f32)/2.0) as i32, ((v.y + 1.0)*(HEIGHT as f32)/2.0) as i32);
            world_coords[j] = v;
        }
        let mut n = (world_coords[2] - world_coords[0]).cross(world_coords[1] - world_coords[0]);
        n = n/dot(n,n).sqrt();
        let intensity = dot(n, LIGHT_DIR);
        if intensity.is_sign_positive() {
            triangle(&screen_coords, &mut image, Rgb([(intensity * 255.0) as u8, (intensity * 255.0) as u8, (intensity * 255.0) as u8]));
        }
    }

    // (0,0) is the bottom left
    imageops::flip_vertical_in_place(&mut image);
    image.save("output.tga").unwrap();
}
