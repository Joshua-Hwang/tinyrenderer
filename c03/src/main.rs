use anyhow::Result;
use cgmath::{dot, Vector2, Vector3};
use image::io::Reader as ImageReader;
use image::{imageops, ImageBuffer, RgbImage};

mod model;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 800;
const LIGHT_DIR: Vector3<f32> = Vector3 {
    x: 0.0,
    y: 0.0,
    z: -1.0,
};

fn barycentric(pts: &[Vector2<f32>; 3], p: Vector2<f32>) -> Vector3<f32> {
    // Let a triangle be labeled ABC which are located at pts[0] pts[1] and pts[2]
    let x = Vector3::new(pts[2].x - pts[0].x, pts[1].x - pts[0].x, pts[0].x - p.x);
    let y = Vector3::new(pts[2].y - pts[0].y, pts[1].y - pts[0].y, pts[0].y - p.y);
    let u = x.cross(y);
    if u.z.abs() < 1.0 {
        Vector3::new(-1.0, 1.0, 1.0)
    } else {
        Vector3::new(1.0 - (u.x + u.y) / u.z, u.y / u.z, u.x / u.z)
    }
}

fn triangle(
    pts: &[Vector3<f32>; 3],
    norm_pts: &[Vector3<f32>; 3],
    uv_pts: &[Vector2<f32>; 3],
    zbuffer: &mut Vec<f32>,
    image: &mut RgbImage,
    texture: &RgbImage,
) {
    let mut bboxmin: Vector2<u32> =
        Vector2::new((image.width() - 1).into(), (image.height() - 1).into());
    let mut bboxmax: Vector2<u32> = Vector2::new(0, 0);
    let clamp: Vector2<u32> = Vector2::new((image.width() - 1).into(), (image.height() - 1).into());
    for i in 0..3 {
        for j in 0..2 {
            if pts[i][j].is_sign_negative() {
                panic!("Triangle outside bounds of canvas");
            }
            bboxmin[j] = bboxmin[j].clamp(0, pts[i][j] as u32);
            bboxmax[j] = bboxmax[j].max(pts[i][j] as u32).min(clamp[j]);
        }
    }
    let pts_2d = pts.map(|pt| Vector2::new(pt.x, pt.y));
    for x in bboxmin.x..=bboxmax.x {
        for y in bboxmin.y..=bboxmax.y {
            let mut p: Vector3<f32> = Vector3::new(x as f32, y as f32, 0.0);
            let bc_screen = barycentric(&pts_2d, Vector2::new(p.x, p.y));
            if bc_screen.x.is_sign_negative()
                || bc_screen.y.is_sign_negative()
                || bc_screen.z.is_sign_negative()
            {
                continue;
            }
            p.z = pts[0].z * bc_screen[0] + pts[1].z * bc_screen[1] + pts[2].z * bc_screen[2];
            let zi = (p.x + p.y * (image.width() as f32)) as usize;
            if zbuffer[zi] < p.z {
                zbuffer[zi] = p.z;

                let mut uv =
                    uv_pts[0] * bc_screen[0] + uv_pts[1] * bc_screen[1] + uv_pts[2] * bc_screen[2];
                uv.x *= texture.width() as f32;
                uv.y *= texture.height() as f32;
                let mut color = texture.get_pixel(uv.x as u32, uv.y as u32).clone();

                let mut n = norm_pts[0] * bc_screen[0]
                    + norm_pts[1] * bc_screen[1]
                    + norm_pts[2] * bc_screen[2];
                n = n / dot(n, n).sqrt();
                let intensity = -dot(n, LIGHT_DIR); // n is wrong way around so swap

                color[0] = ((color[0] as f32) * intensity) as u8;
                color[1] = ((color[1] as f32) * intensity) as u8;
                color[2] = ((color[2] as f32) * intensity) as u8;
                image.put_pixel(p.x as u32, p.y as u32, color);
            }
        }
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let model = model::file_to_model(if args.len() == 2 {
        &args[1]
    } else {
        "obj/african_head.obj"
    })?;
    let mut texture = ImageReader::open("obj/african_head_diffuse.tga")?
        .decode()?
        .to_rgb8();
    imageops::flip_vertical_in_place(&mut texture);

    let mut image: RgbImage = ImageBuffer::new(WIDTH, HEIGHT);
    let mut zbuffer: Vec<f32> = vec![f32::NEG_INFINITY; (WIDTH * HEIGHT).try_into()?];

    let verts = model.get_verts();
    let norms = model.get_norms();
    let uvs = model.get_uvs();
    for face in model.get_faces() {
        let mut screen_coords: [Vector3<f32>; 3] = [Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }; 3];
        let mut norm_coords: [Vector3<f32>; 3] = [Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }; 3];
        let mut world_coords: [Vector3<f32>; 3] = [Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }; 3];
        let mut texture_coords: [Vector2<f32>; 3] = [Vector2 { x: 0.0, y: 0.0 }; 3];
        for j in 0..3usize {
            let v = verts[face[j].v];
            screen_coords[j] = Vector3::new(
                (v.x + 1.0) * (WIDTH as f32) / 2.0,
                (v.y + 1.0) * (HEIGHT as f32) / 2.0,
                v.z,
            );
            norm_coords[j] = norms[face[j].v];
            // no need for normalization since they already are
            texture_coords[j] = uvs[face[j].vt];
            world_coords[j] = v;
        }
        triangle(
            &screen_coords,
            &norm_coords,
            &texture_coords,
            &mut zbuffer,
            &mut image,
            &texture,
        );
    }

    // (0,0) is the bottom left
    imageops::flip_vertical_in_place(&mut image);
    image.save("output.tga")?;

    Ok(())
}
