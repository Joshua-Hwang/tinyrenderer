extern crate image;

use image::{Rgb, ImageBuffer, RgbImage, imageops};

const WHITE: Rgb<u8> = Rgb([255, 255, 255]);
const RED: Rgb<u8> = Rgb([255, 0, 0]);

fn main() {
    let mut image: RgbImage = ImageBuffer::new(100, 100);
    image.put_pixel(52, 41, RED);
    imageops::flip_vertical_in_place(&mut image);
    image.save("output.tga").unwrap();
}
