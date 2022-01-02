use super::model;
use super::our_gl;
use cgmath::{
    dot, InnerSpace, Matrix, Matrix3, Matrix4, SquareMatrix, Transform, Vector2, Vector3, Vector4,
};
use image::{GrayImage, Rgb, RgbImage};

const WIGGLE: f32 = 5.0; // magic number to avoid z-fighting

pub struct GouraudShader {
    varying_intensity: Vector3<f32>,
    light_dir: Vector3<f32>,
}

impl GouraudShader {
    pub const fn new(light_dir: Vector3<f32>) -> GouraudShader {
        GouraudShader {
            light_dir,
            varying_intensity: Vector3::<f32>::new(0.0, 0.0, 0.0),
        }
    }
}

impl our_gl::Shader for GouraudShader {
    fn vertex(
        &mut self,
        model: &model::Model,
        iface: usize,
        nthvert: usize,
        mat: Matrix4<f32>,
    ) -> Vector4<f32> {
        let v = model.get_faces()[iface][nthvert].v;
        let n = model.get_norms()[v];
        self.varying_intensity[nthvert] = dot(n, self.light_dir.normalize()).max(0.0);

        let gl_vertex = model.get_verts()[v].extend(1.0);
        mat * gl_vertex
    }

    fn fragment(&self, bc: Vector3<f32>, color: &mut Rgb<u8>) -> bool {
        let intensity = dot(self.varying_intensity, bc);
        color[0] = (255.0 * intensity) as u8;
        color[1] = (255.0 * intensity) as u8;
        color[2] = (255.0 * intensity) as u8;
        true
    }
}

pub struct FunnyShader {
    varying_intensity: Vector3<f32>,
    light_dir: Vector3<f32>,
}

impl FunnyShader {
    pub const fn new(light_dir: Vector3<f32>) -> FunnyShader {
        FunnyShader {
            light_dir,
            varying_intensity: Vector3::<f32>::new(0.0, 0.0, 0.0),
        }
    }
}

impl our_gl::Shader for FunnyShader {
    fn vertex(
        &mut self,
        model: &model::Model,
        iface: usize,
        nthvert: usize,
        mat: Matrix4<f32>,
    ) -> Vector4<f32> {
        let v = model.get_faces()[iface][nthvert].v;
        let n = model.get_norms()[v];
        self.varying_intensity[nthvert] = dot(n, self.light_dir.normalize()).max(0.0);

        let gl_vertex = model.get_verts()[v].extend(1.0);
        mat * gl_vertex
    }

    fn fragment(&self, bc: Vector3<f32>, color: &mut Rgb<u8>) -> bool {
        let mut intensity = dot(self.varying_intensity, bc);
        if intensity > 0.85 {
            intensity = 1.00;
        } else if intensity > 0.6 {
            intensity = 0.80;
        } else if intensity > 0.45 {
            intensity = 0.60;
        } else if intensity > 0.30 {
            intensity = 0.45;
        } else if intensity > 0.15 {
            intensity = 0.30;
        } else {
            intensity = 0.0;
        }
        color[0] = (255.0 * intensity) as u8;
        color[1] = (155.0 * intensity) as u8;
        color[2] = (0.0 * intensity) as u8;
        true
    }
}

pub struct TextureShader {
    light_dir: Vector3<f32>,
    texture: RgbImage,
    varying_intensity: Vector3<f32>,
    varying_uv: [Vector2<f32>; 3],
}

impl TextureShader {
    pub const fn new(light_dir: Vector3<f32>, texture: RgbImage) -> TextureShader {
        TextureShader {
            light_dir,
            texture,
            varying_intensity: Vector3::<f32>::new(0.0, 0.0, 0.0),
            varying_uv: [Vector2 { x: 0.0, y: 0.0 }; 3],
        }
    }
}

impl our_gl::Shader for TextureShader {
    fn vertex(
        &mut self,
        model: &model::Model,
        iface: usize,
        nthvert: usize,
        mat: Matrix4<f32>,
    ) -> Vector4<f32> {
        let v = model.get_faces()[iface][nthvert].v;
        let vt = model.get_faces()[iface][nthvert].vt;

        let n = model.get_norms()[v];
        self.varying_intensity[nthvert] = dot(n, self.light_dir.normalize()).max(0.0);

        self.varying_uv[nthvert] = model.get_uvs()[vt];

        let gl_vertex = model.get_verts()[v].extend(1.0);
        mat * gl_vertex
    }

    fn fragment(&self, bc: Vector3<f32>, color: &mut Rgb<u8>) -> bool {
        let mut uv =
            self.varying_uv[0] * bc[0] + self.varying_uv[1] * bc[1] + self.varying_uv[2] * bc[2];
        uv.x *= self.texture.width() as f32;
        uv.y *= self.texture.height() as f32;
        *color = self.texture.get_pixel(uv.x as u32, uv.y as u32).clone();

        let intensity = dot(self.varying_intensity, bc);
        color[0] = (color[0] as f32 * intensity) as u8;
        color[1] = (color[1] as f32 * intensity) as u8;
        color[2] = (color[2] as f32 * intensity) as u8;
        true
    }
}

pub struct NormalShader {
    light_dir: Vector3<f32>,
    texture: RgbImage,
    normal_map: RgbImage,
    varying_uv: [Vector2<f32>; 3],
    varying_tri: [Vector4<f32>; 3],
    ndc_tri: [Vector3<f32>; 3], // normalized version of above
    varying_norm: [Vector3<f32>; 3],
    uniform_m: Matrix4<f32>,
    uniform_mit: Matrix4<f32>, // invert_transpose of m
}

impl NormalShader {
    pub fn new(
        light_dir: Vector3<f32>,
        texture: RgbImage,
        normal_map: RgbImage,
        uniform_m: Matrix4<f32>, // projection * model_view
    ) -> NormalShader {
        NormalShader {
            light_dir: (uniform_m * light_dir.extend(0.0)).truncate().normalize(),
            texture,
            normal_map,
            varying_uv: [Vector2 { x: 0.0, y: 0.0 }; 3],
            varying_tri: [Vector4 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                w: 0.0,
            }; 3],
            ndc_tri: [Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            }; 3],
            varying_norm: [Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            }; 3],
            uniform_m,
            uniform_mit: uniform_m
                .inverse_transform()
                .expect("Could not find inverse")
                .transpose(),
        }
    }
}

impl our_gl::Shader for NormalShader {
    fn vertex(
        &mut self,
        model: &model::Model,
        iface: usize,
        nthvert: usize,
        mat: Matrix4<f32>,
    ) -> Vector4<f32> {
        let v = model.get_faces()[iface][nthvert].v;
        let vt = model.get_faces()[iface][nthvert].vt;

        self.varying_uv[nthvert] = model.get_uvs()[vt];
        self.varying_norm[nthvert] =
            (self.uniform_mit * model.get_norms()[v].extend(0.0)).truncate();

        let gl_vertex = model.get_verts()[v].extend(1.0);
        self.varying_tri[nthvert] = gl_vertex;
        self.ndc_tri[nthvert] = gl_vertex.truncate() / gl_vertex.w;
        mat * gl_vertex
    }

    fn fragment(&self, bc: Vector3<f32>, color: &mut Rgb<u8>) -> bool {
        let bn = (self.varying_norm[0] * bc[0]
            + self.varying_norm[1] * bc[1]
            + self.varying_norm[2] * bc[2])
            .normalize();
        let uv =
            self.varying_uv[0] * bc[0] + self.varying_uv[1] * bc[1] + self.varying_uv[2] * bc[2];
        *color = self
            .texture
            .get_pixel(
                (uv.x * self.texture.width() as f32) as u32,
                (uv.y * self.texture.height() as f32) as u32,
            )
            .clone();

        let a = Matrix3::<f32>::from_cols(
            self.ndc_tri[1] - self.ndc_tri[0],
            self.ndc_tri[2] - self.ndc_tri[0],
            bn,
        )
        .transpose();
        let ai = a.invert().expect("Matrix A does not have an inverse");

        let i = ai
            * Vector3::<f32>::new(
                self.varying_uv[1].x - self.varying_uv[0].x,
                self.varying_uv[2].x - self.varying_uv[0].x,
                0.0,
            );
        let j = ai
            * Vector3::<f32>::new(
                self.varying_uv[1].y - self.varying_uv[0].y,
                self.varying_uv[2].y - self.varying_uv[0].y,
                0.0,
            );

        let b = Matrix3::<f32>::from_cols(i.normalize(), j.normalize(), bn);

        let n_info = self.normal_map.get_pixel(
            (uv.x * self.normal_map.width() as f32) as u32,
            (uv.y * self.normal_map.height() as f32) as u32,
        );
        let n = b * Vector3::<f32>::new(
            n_info[0] as f32 / 255.0 * 2.0 - 1.0,
            n_info[1] as f32 / 255.0 * 2.0 - 1.0,
            n_info[2] as f32 / 255.0 * 2.0 - 1.0,
        )
        .normalize();
        let intensity = f32::max(0.0, dot(n, self.light_dir));
        color[0] = (color[0] as f32 * intensity) as u8;
        color[1] = (color[1] as f32 * intensity) as u8;
        color[2] = (color[2] as f32 * intensity) as u8;
        true
    }
}

pub struct SpecularShader {
    light_dir: Vector3<f32>,
    texture: RgbImage,
    normal_map: RgbImage,
    specular_map: GrayImage,
    varying_uv: [Vector2<f32>; 3],
    varying_tri: [Vector4<f32>; 3],
    ndc_tri: [Vector3<f32>; 3], // normalized version of above
    varying_norm: [Vector3<f32>; 3],
    uniform_mit: Matrix4<f32>, // invert_transpose of m
}

impl SpecularShader {
    pub fn new(
        light_dir: Vector3<f32>,
        texture: RgbImage,
        normal_map: RgbImage,
        specular_map: GrayImage,
        uniform_m: Matrix4<f32>, // projection * model_view
    ) -> SpecularShader {
        SpecularShader {
            light_dir: (uniform_m * light_dir.extend(0.0)).truncate().normalize(),
            texture,
            normal_map,
            specular_map,
            varying_uv: [Vector2 { x: 0.0, y: 0.0 }; 3],
            varying_tri: [Vector4 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                w: 0.0,
            }; 3],
            ndc_tri: [Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            }; 3],
            varying_norm: [Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            }; 3],
            uniform_mit: uniform_m
                .inverse_transform()
                .expect("Could not find inverse")
                .transpose(),
        }
    }
}

impl our_gl::Shader for SpecularShader {
    fn vertex(
        &mut self,
        model: &model::Model,
        iface: usize,
        nthvert: usize,
        mat: Matrix4<f32>,
    ) -> Vector4<f32> {
        let v = model.get_faces()[iface][nthvert].v;
        let vt = model.get_faces()[iface][nthvert].vt;

        self.varying_uv[nthvert] = model.get_uvs()[vt];
        self.varying_norm[nthvert] =
            (self.uniform_mit * model.get_norms()[v].extend(0.0)).truncate();

        let gl_vertex = model.get_verts()[v].extend(1.0);
        self.varying_tri[nthvert] = gl_vertex;
        self.ndc_tri[nthvert] = gl_vertex.truncate() / gl_vertex.w;
        mat * gl_vertex
    }

    fn fragment(&self, bc: Vector3<f32>, color: &mut Rgb<u8>) -> bool {
        let bn = (self.varying_norm[0] * bc[0]
            + self.varying_norm[1] * bc[1]
            + self.varying_norm[2] * bc[2])
            .normalize();
        let uv =
            self.varying_uv[0] * bc[0] + self.varying_uv[1] * bc[1] + self.varying_uv[2] * bc[2];
        *color = self
            .texture
            .get_pixel(
                (uv.x * self.texture.width() as f32) as u32,
                (uv.y * self.texture.height() as f32) as u32,
            )
            .clone();

        let a = Matrix3::<f32>::from_cols(
            self.ndc_tri[1] - self.ndc_tri[0],
            self.ndc_tri[2] - self.ndc_tri[0],
            bn,
        )
        .transpose();
        let ai = a.invert().expect("Matrix A does not have an inverse");

        let i = ai
            * Vector3::<f32>::new(
                self.varying_uv[1].x - self.varying_uv[0].x,
                self.varying_uv[2].x - self.varying_uv[0].x,
                0.0,
            );
        let j = ai
            * Vector3::<f32>::new(
                self.varying_uv[1].y - self.varying_uv[0].y,
                self.varying_uv[2].y - self.varying_uv[0].y,
                0.0,
            );

        let b = Matrix3::<f32>::from_cols(i.normalize(), j.normalize(), bn);

        let n_info = self.normal_map.get_pixel(
            (uv.x * self.normal_map.width() as f32) as u32,
            (uv.y * self.normal_map.height() as f32) as u32,
        );
        let n = b * Vector3::<f32>::new(
            n_info[0] as f32 / 255.0 * 2.0 - 1.0,
            n_info[1] as f32 / 255.0 * 2.0 - 1.0,
            n_info[2] as f32 / 255.0 * 2.0 - 1.0,
        )
        .normalize();

        // since number is <= 1 raising to the power sends < 1 to 0
        let spec_pow = self.specular_map.get_pixel(
            (uv.x * self.specular_map.width() as f32) as u32,
            (uv.y * self.specular_map.height() as f32) as u32,
        )[0];

        let r = (n * (2.0 * dot(n, self.light_dir)) - self.light_dir).normalize();
        let spec = r.z.max(0.0).powf(spec_pow as f32);
        let diff = f32::max(0.0, dot(n, self.light_dir));
        color[0] = (5.0 + color[0] as f32 * (diff + 0.3 * spec)).min(255.0) as u8;
        color[1] = (5.0 + color[1] as f32 * (diff + 0.3 * spec)).min(255.0) as u8;
        color[2] = (5.0 + color[2] as f32 * (diff + 0.3 * spec)).min(255.0) as u8;
        true
    }
}

pub struct DepthShader {
    varying_tri: [Vector3<f32>; 3],
}

impl DepthShader {
    pub fn new() -> DepthShader {
        DepthShader {
            varying_tri: [Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            }; 3],
        }
    }
}

impl our_gl::Shader for DepthShader {
    fn vertex(
        &mut self,
        model: &model::Model,
        iface: usize,
        nthvert: usize,
        mat: Matrix4<f32>,
    ) -> Vector4<f32> {
        let v = model.get_faces()[iface][nthvert].v;
        let gl_vertex = mat * model.get_verts()[v].extend(1.0);
        self.varying_tri[nthvert] = gl_vertex.truncate() / gl_vertex.w;
        gl_vertex
    }

    fn fragment(&self, bc: Vector3<f32>, color: &mut Rgb<u8>) -> bool {
        let p =
            self.varying_tri[0] * bc[0] + self.varying_tri[1] * bc[1] + self.varying_tri[2] * bc[2];
        let depth: u8 = (255.0 * p.z / our_gl::DEPTH) as u8;
        color[0] = depth;
        color[1] = depth;
        color[2] = depth;
        true
    }
}

pub struct ShadowShader {
    light_dir: Vector3<f32>,
    texture: RgbImage,
    normal_map: RgbImage,
    specular_map: GrayImage,
    varying_uv: [Vector2<f32>; 3],
    varying_tri: [Vector4<f32>; 3],
    ndc_tri: [Vector3<f32>; 3], // normalized version of above
    varying_norm: [Vector3<f32>; 3],
    uniform_m: Matrix4<f32>,
    uniform_mit: Matrix4<f32>, // invert_transpose of m
    uniform_m_shadow: Matrix4<f32>,
    shadow_buffer: GrayImage,
}

impl ShadowShader {
    pub fn new(
        light_dir: Vector3<f32>,
        texture: RgbImage,
        normal_map: RgbImage,
        specular_map: GrayImage,
        uniform_m: Matrix4<f32>, // projection * model_view
        uniform_m_shadow: Matrix4<f32>,
        shadow_buffer: GrayImage,
    ) -> ShadowShader {
        ShadowShader {
            light_dir: (uniform_m * light_dir.extend(0.0)).truncate().normalize(),
            texture,
            normal_map,
            specular_map,
            varying_uv: [Vector2 { x: 0.0, y: 0.0 }; 3],
            varying_tri: [Vector4 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                w: 0.0,
            }; 3],
            ndc_tri: [Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            }; 3],
            varying_norm: [Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            }; 3],
            uniform_m,
            uniform_mit: uniform_m
                .inverse_transform()
                .expect("Could not find inverse")
                .transpose(),
            uniform_m_shadow,
            shadow_buffer,
        }
    }
}

impl our_gl::Shader for ShadowShader {
    fn vertex(
        &mut self,
        model: &model::Model,
        iface: usize,
        nthvert: usize,
        mat: Matrix4<f32>,
    ) -> Vector4<f32> {
        let v = model.get_faces()[iface][nthvert].v;
        let vt = model.get_faces()[iface][nthvert].vt;

        self.varying_uv[nthvert] = model.get_uvs()[vt];
        self.varying_norm[nthvert] =
            (self.uniform_mit * model.get_norms()[v].extend(0.0)).truncate();

        let gl_vertex = mat * model.get_verts()[v].extend(1.0);
        self.varying_tri[nthvert] = gl_vertex;
        self.ndc_tri[nthvert] = gl_vertex.truncate() / gl_vertex.w;
        gl_vertex
    }

    fn fragment(&self, bc: Vector3<f32>, color: &mut Rgb<u8>) -> bool {
        let sb_p4 = self.uniform_m_shadow
            * (self.ndc_tri[0] * bc[0] + self.ndc_tri[1] * bc[1] + self.ndc_tri[2] * bc[2])
                .extend(1.0);
        let sb_p = sb_p4.truncate() / sb_p4.w;
        let shadow = if (self.shadow_buffer.get_pixel(sb_p.x as u32, sb_p.y as u32)[0] as f32)
            .lt(&(sb_p.z + WIGGLE))
        {
            1.0
        } else {
            0.3
        };

        let bn = (self.varying_norm[0] * bc[0]
            + self.varying_norm[1] * bc[1]
            + self.varying_norm[2] * bc[2])
            .normalize();
        let uv =
            self.varying_uv[0] * bc[0] + self.varying_uv[1] * bc[1] + self.varying_uv[2] * bc[2];
        *color = self
            .texture
            .get_pixel(
                (uv.x * self.texture.width() as f32) as u32,
                (uv.y * self.texture.height() as f32) as u32,
            )
            .clone();

        let a = Matrix3::<f32>::from_cols(
            self.ndc_tri[1] - self.ndc_tri[0],
            self.ndc_tri[2] - self.ndc_tri[0],
            bn,
        )
        .transpose();
        let ai = a.invert().expect("Matrix A does not have an inverse");

        let i = ai
            * Vector3::<f32>::new(
                self.varying_uv[1].x - self.varying_uv[0].x,
                self.varying_uv[2].x - self.varying_uv[0].x,
                0.0,
            );
        let j = ai
            * Vector3::<f32>::new(
                self.varying_uv[1].y - self.varying_uv[0].y,
                self.varying_uv[2].y - self.varying_uv[0].y,
                0.0,
            );

        let b = Matrix3::<f32>::from_cols(i.normalize(), j.normalize(), bn);

        let n_info = self.normal_map.get_pixel(
            (uv.x * self.normal_map.width() as f32) as u32,
            (uv.y * self.normal_map.height() as f32) as u32,
        );
        let n = b * Vector3::<f32>::new(
            n_info[0] as f32 / 255.0 * 2.0 - 1.0,
            n_info[1] as f32 / 255.0 * 2.0 - 1.0,
            n_info[2] as f32 / 255.0 * 2.0 - 1.0,
        )
        .normalize();

        // since number is <= 1 raising to the power sends < 1 to 0
        let spec_pow = self.specular_map.get_pixel(
            (uv.x * self.specular_map.width() as f32) as u32,
            (uv.y * self.specular_map.height() as f32) as u32,
        )[0];

        let r = (n * (2.0 * dot(n, self.light_dir)) - self.light_dir).normalize();
        let spec = r.z.max(0.0).powf(spec_pow as f32);
        let diff = f32::max(0.0, dot(n, self.light_dir));
        color[0] = (20.0 + color[0] as f32 * shadow * (1.2 * diff + 0.6 * spec)).min(255.0) as u8;
        color[1] = (20.0 + color[1] as f32 * shadow * (1.2 * diff + 0.6 * spec)).min(255.0) as u8;
        color[2] = (20.0 + color[2] as f32 * shadow * (1.2 * diff + 0.6 * spec)).min(255.0) as u8;
        true
    }
}

pub struct ZShader {
    pub varying_tri: [Vector4<f32>; 3],
}

impl ZShader {
    pub fn new() -> ZShader {
        ZShader {
            varying_tri: [Vector4 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                w: 0.0,
            }; 3],
        }
    }
}

impl our_gl::Shader for ZShader {
    fn vertex(
        &mut self,
        model: &model::Model,
        iface: usize,
        nthvert: usize,
        mat: Matrix4<f32>,
    ) -> Vector4<f32> {
        let v = model.get_faces()[iface][nthvert].v;
        let gl_vertex = mat * model.get_verts()[v].extend(1.0);
        self.varying_tri[nthvert] = gl_vertex;
        gl_vertex
    }

    fn fragment(&self, _bc: Vector3<f32>, color: &mut Rgb<u8>) -> bool {
        *color = Rgb([0, 0, 0]);
        true
    }
}
