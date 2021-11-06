use std::fs;
use std::io::{ Error, ErrorKind };
use anyhow::Result;
use cgmath::Vector3;

#[derive(Debug)]
pub struct Model {
    verts: Vec<Vector3<f32>>,
    faces: Vec<Vec<usize>>,
}

impl Model {
    pub fn get_verts(&self) -> &Vec<Vector3<f32>> { &self.verts }
    pub fn get_faces(&self) -> &Vec<Vec<usize>> { &self.faces }
}

pub fn file_to_model(filename: &str) -> Result<Model> {
    let mut model = Model {
        verts: Vec::new(),
        faces: Vec::new(),
    };

    let obj = fs::read_to_string(filename)?;
    for l in obj.lines() {
        if l.starts_with("v ") {
            let mut iter = l.split_ascii_whitespace();
            iter.next(); // drop first character
            let v = Vector3::new(
                iter.next().ok_or(Error::new(ErrorKind::InvalidData, "obj file 'v' line malformed"))?.parse::<f32>()?,
                iter.next().ok_or(Error::new(ErrorKind::InvalidData, "obj file 'v' line malformed"))?.parse::<f32>()?,
                iter.next().ok_or(Error::new(ErrorKind::InvalidData, "obj file 'v' line malformed"))?.parse::<f32>()?
                );
            model.verts.push(v);
        } else if l.starts_with("f ") {
            let mut f: Vec<usize> = Vec::new();
            let mut iter = l.split_ascii_whitespace();
            iter.next(); // drop first character
            for ss in iter {
                f.push(ss.split_once('/').ok_or(Error::new(ErrorKind::InvalidData, "obj file 'f' line malformed"))?.0.parse::<usize>()? - 1);
            }
            model.faces.push(f);
        }
    }

    Ok(model)
}
