use std::mem;
use std::cmp;
use image::{Rgb, ImageBuffer, RgbImage, imageops};

mod model;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 800;
const WHITE: Rgb<u8> = Rgb([255, 255, 255]);

fn line(mut x0: u32, mut y0: u32, mut x1: u32, mut y1: u32, image: &mut RgbImage, color: Rgb<u8>) {
    let steep = if (x0 as i32 - x1 as i32).abs() < (y0 as i32 - y1 as i32).abs() {
        mem::swap(&mut x0, &mut y0);
        mem::swap(&mut x1, &mut y1);
        true
    } else {
        false
    };

    // guaranteed that x0 >= x1 but the ys have no gurantee
    if x0 > x1 {
        mem::swap(&mut x0, &mut x1);
        mem::swap(&mut y0, &mut y1);
    }

    let dx = x1 - x0;
    let dy = y1 as i32 - y0 as i32;
    let derror2 = dy.abs() * 2;
    let mut error2 = 0;
    let mut y = y0;
    for x in x0..=x1 {
        if steep {
            image.put_pixel(y, x, color);
        } else {
            image.put_pixel(x, y, color);
        }
        error2 += derror2;
        if error2 > dx as i32 {
            y = if y1 > y0 { y + 1 } else { y - 1 };
            error2 -= dx as i32 * 2;
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let model = model::file_to_model(if args.len() == 2 { &args[1] } else { "obj/african_head.obj" }).unwrap();

    let mut image: RgbImage = ImageBuffer::new(WIDTH, HEIGHT);
    let verts = model.get_verts();
    for face in model.get_faces() {
        for j in 0..3usize {
            let v0 = verts[face[j]];
            let v1 = verts[face[(j+1)%3]];
            let x0 = cmp::min(((v0.x+1.0)*(WIDTH as f32)/2.0) as u32, WIDTH - 1);
            let y0 = cmp::min(((v0.y+1.0)*(HEIGHT as f32)/2.0) as u32, HEIGHT - 1);
            let x1 = cmp::min(((v1.x+1.0)*(WIDTH as f32)/2.0) as u32, WIDTH - 1);
            let y1 = cmp::min(((v1.y+1.0)*(HEIGHT as f32)/2.0) as u32, HEIGHT - 1);
            line(x0, y0, x1, y1, &mut image, WHITE);
        }
    }

    // (0,0) is the bottom left
    imageops::flip_vertical_in_place(&mut image);
    image.save("output.tga").unwrap();
}
