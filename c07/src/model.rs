use anyhow::Result;
use cgmath::{InnerSpace, Vector2, Vector3};
use std::fs;
use std::io::{Error, ErrorKind};

#[derive(Debug)]
pub struct VertexInfo {
    pub v: usize,
    pub vt: usize,
}

#[derive(Debug)]
pub struct Model {
    verts: Vec<Vector3<f32>>, // access specific norms via VertexInfo.v
    norms: Vec<Vector3<f32>>, // access specific norms via VertexInfo.v
    uvs: Vec<Vector2<f32>>,
    faces: Vec<Vec<VertexInfo>>,
}

impl Model {
    pub fn get_verts(&self) -> &Vec<Vector3<f32>> {
        &self.verts
    }
    pub fn get_faces(&self) -> &Vec<Vec<VertexInfo>> {
        &self.faces
    }
    pub fn get_uvs(&self) -> &Vec<Vector2<f32>> {
        &self.uvs
    }
    pub fn get_norms(&self) -> &Vec<Vector3<f32>> {
        &self.norms
    }
}

pub fn file_to_model(filename: &str) -> Result<Model> {
    let mut model = Model {
        verts: Vec::new(),
        norms: Vec::new(),
        faces: Vec::new(),
        uvs: Vec::new(),
    };

    let obj = fs::read_to_string(filename)?;
    for l in obj.lines() {
        if l.starts_with("v ") {
            let mut iter = l.split_ascii_whitespace();
            iter.next(); // drop first character
            let v = Vector3::new(
                iter.next()
                    .ok_or(Error::new(
                        ErrorKind::InvalidData,
                        "obj file 'v' line malformed",
                    ))?
                    .parse::<f32>()?,
                iter.next()
                    .ok_or(Error::new(
                        ErrorKind::InvalidData,
                        "obj file 'v' line malformed",
                    ))?
                    .parse::<f32>()?,
                iter.next()
                    .ok_or(Error::new(
                        ErrorKind::InvalidData,
                        "obj file 'v' line malformed",
                    ))?
                    .parse::<f32>()?,
            );
            model.verts.push(v);
        } else if l.starts_with("f ") {
            let mut f: Vec<VertexInfo> = Vec::new();
            let mut iter = l.split_ascii_whitespace();
            iter.next(); // drop first character
            for ss in iter {
                let mut sss = ss.split('/');
                let v = sss
                    .next()
                    .ok_or(Error::new(
                        ErrorKind::InvalidData,
                        "obj file 'f' line malformed",
                    ))?
                    .parse::<usize>()?
                    - 1;
                let vt = sss
                    .next()
                    .ok_or(Error::new(
                        ErrorKind::InvalidData,
                        "obj file 'f' line malformed",
                    ))?
                    .parse::<usize>()?
                    - 1;
                f.push(VertexInfo { v, vt });
            }
            model.faces.push(f);
        } else if l.starts_with("vt ") {
            let mut iter = l.split_ascii_whitespace();
            iter.next(); // drop first portion
            let uv = Vector2::new(
                iter.next()
                    .ok_or(Error::new(
                        ErrorKind::InvalidData,
                        "obj file 'v' line malformed",
                    ))?
                    .parse::<f32>()?,
                iter.next()
                    .ok_or(Error::new(
                        ErrorKind::InvalidData,
                        "obj file 'v' line malformed",
                    ))?
                    .parse::<f32>()?,
            );
            model.uvs.push(uv);
        } else if l.starts_with("vn ") {
            let mut iter = l.split_ascii_whitespace();
            iter.next(); // drop first character
            let v = Vector3::new(
                iter.next()
                    .ok_or(Error::new(
                        ErrorKind::InvalidData,
                        "obj file 'v' line malformed",
                    ))?
                    .parse::<f32>()?,
                iter.next()
                    .ok_or(Error::new(
                        ErrorKind::InvalidData,
                        "obj file 'v' line malformed",
                    ))?
                    .parse::<f32>()?,
                iter.next()
                    .ok_or(Error::new(
                        ErrorKind::InvalidData,
                        "obj file 'v' line malformed",
                    ))?
                    .parse::<f32>()?,
            );
            model.norms.push(v.normalize());
        }
    }

    Ok(model)
}
