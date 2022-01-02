mod model;
mod our_gl;
mod shaders;

use anyhow::Result;
use cgmath::{InnerSpace, Transform, Vector3, Vector4};
use image::io::Reader as ImageReader;
use image::{imageops, GrayImage, ImageBuffer, RgbImage};
use our_gl::Shader;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 800;
const EYE: Vector3<f32> = Vector3 {
    x: 1.0,
    y: 0.0,
    z: 2.0,
};
const CENTER: Vector3<f32> = Vector3 {
    x: 0.0,
    y: 0.0,
    z: 0.0,
};
const UP: Vector3<f32> = Vector3 {
    x: 0.0,
    y: 1.0,
    z: 0.0,
};

const LIGHT_DIR: Vector3<f32> = Vector3 {
    x: -1.0,
    y: -1.0,
    z: 2.0,
};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let path = if args.len() == 2 {
        &args[1]
    } else {
        "obj/african_head/african_head"
    };
    let model = model::file_to_model(format!("{}.obj", path).as_str())?;
    let mut texture = ImageReader::open(format!("{}_diffuse.tga", path).as_str())?
        .decode()?
        .to_rgb8();
    imageops::flip_vertical_in_place(&mut texture);

    let mut normal_map = ImageReader::open(format!("{}_nm_tangent.tga", path).as_str())?
        .decode()?
        .to_rgb8();
    imageops::flip_vertical_in_place(&mut normal_map);

    let mut specular_map = ImageReader::open(format!("{}_spec.tga", path).as_str())?
        .decode()?
        .to_luma8();
    imageops::flip_vertical_in_place(&mut specular_map);

    let mut image: RgbImage = ImageBuffer::new(WIDTH, HEIGHT);
    let mut zbuffer: GrayImage = ImageBuffer::new(WIDTH, HEIGHT);

    let mut shadow_buffer: GrayImage = ImageBuffer::new(WIDTH, HEIGHT);
    let m = {
        // rendering the shadow buffer
        let mut depth: RgbImage = ImageBuffer::new(WIDTH, HEIGHT);

        let model_view = our_gl::lookat(LIGHT_DIR, CENTER, UP);
        let viewport = our_gl::viewport(
            (WIDTH / 8) as f32,
            (HEIGHT / 8) as f32,
            (WIDTH * 3 / 4) as f32,
            (HEIGHT * 3 / 4) as f32,
        );
        let projection = our_gl::projection(0.0);
        let mat = viewport * projection * model_view;

        let mut depth_shader = shaders::DepthShader::new();
        for i in 0..model.get_faces().len() {
            let mut screen_coords: [Vector4<f32>; 3] = [Vector4 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                w: 0.0,
            }; 3];
            for j in 0..3usize {
                screen_coords[j] = depth_shader.vertex(&model, i, j, mat);
            }
            our_gl::triangle(
                &screen_coords,
                &depth_shader,
                &mut depth,
                &mut shadow_buffer,
            );
        }

        imageops::flip_vertical_in_place(&mut depth);
        depth.save("depth.tga")?;

        // imageops::flip_vertical_in_place(&mut shadow_buffer);
        // shadow_buffer.save("shadow_buffer.tga")?;
        mat
    };

    {
        // ambient occlusion
        let model_view = our_gl::lookat(EYE, CENTER, UP);
        let viewport = our_gl::viewport(
            (WIDTH / 8) as f32,
            (HEIGHT / 8) as f32,
            (WIDTH * 3 / 4) as f32,
            (HEIGHT * 3 / 4) as f32,
        );
        let projection = our_gl::projection(-1.0 / (EYE - CENTER).magnitude());
        let mat = viewport * projection * model_view;

        let mut z_shader = shaders::ZShader::new();
        for i in 0..model.get_faces().len() {
            for j in 0..3usize {
                z_shader.vertex(&model, i, j, mat);
            }
            // first argument is not used
            //our_gl::triangle(&z_shader.varying_tri, &z_shader, &mut image, &mut zbuffer);
        }
    }

    {
        // rendering the frame buffer
        let model_view = our_gl::lookat(EYE, CENTER, UP);
        let viewport = our_gl::viewport(
            (WIDTH / 8) as f32,
            (HEIGHT / 8) as f32,
            (WIDTH * 3 / 4) as f32,
            (HEIGHT * 3 / 4) as f32,
        );
        let projection = our_gl::projection(-1.0 / (EYE - CENTER).magnitude());

        let mat = viewport * projection * model_view;

        let mut shader = shaders::ShadowShader::new(
            LIGHT_DIR.normalize(),
            texture,
            normal_map,
            specular_map,
            projection * model_view,
            m * mat.inverse_transform().expect("mat has not inverse"),
            shadow_buffer,
        );

        for i in 0..model.get_faces().len() {
            let mut screen_coords: [Vector4<f32>; 3] = [Vector4 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                w: 0.0,
            }; 3];
            for j in 0..3usize {
                screen_coords[j] = shader.vertex(&model, i, j, mat);
            }
            our_gl::triangle(&screen_coords, &shader, &mut image, &mut zbuffer);
        }

        // (0,0) is the bottom left
        imageops::flip_vertical_in_place(&mut image);
        image.save("output.tga")?;
        // imageops::flip_vertical_in_place(&mut zbuffer);
        // zbuffer.save("debug.tga")?;
    }

    Ok(())
}
