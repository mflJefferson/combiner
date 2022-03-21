mod args;

use std::fs::File;
use std::io::BufReader;
use std::os::raw::c_double;
use std::sync::Arc;
use image::{DynamicImage, GenericImageView, ImageError, ImageFormat};
use image::imageops::{FilterType};
use image::io::Reader;
use args::Args;
use crate::ImageDataErros::{UnableToDecodeImage, UnableToSaveImage};

#[derive(Debug)]
enum ImageDataErros {
    DifferentImageFormats,
    BufferTooSmall,
    UnableToReadImageFromPath(std::io::Error),
    UnableToDecodeImage(ImageError),
    UnableToSaveImage(ImageError),
    UnableToFormatImage(String)
}

struct FloatingImage {
    width: u32,
    height: u32,
    data: Vec<u8>,
    name: String,
}

impl FloatingImage {
    fn new(width: u32, height: u32, name: String) -> Self {
        let buffer_capacity = height * width * 4;
        let buffer = Vec::with_capacity(buffer_capacity.try_into().unwrap());
        FloatingImage {
            width,
            height,
            data: buffer,
            name
        }
    }
    fn set_data(&mut self, data: Vec<u8>) -> Result<(), ImageDataErros> {
        if data.len() > self.data.capacity() {
            return Err(ImageDataErros::BufferTooSmall);
        }
        self.data = data;
        Ok(())
    }

}

fn main() -> Result<(), ImageDataErros> {
    let args = Args::new();
    let (image_1, image_format_1) = find_image_from_path(args.image_1)?;
    let (image_2, image_format_2) = find_image_from_path(args.image_2)?;

    if image_format_1 != image_format_2{
        return Err(ImageDataErros::DifferentImageFormats);
    }

    let (image_1, image_2) = standardise_size(image_1, image_2);
    let mut output = FloatingImage::new(image_1.width(), image_1.height(), args.output);
    let combined_data = combine_images(image_1, image_2);
    output.set_data(combined_data)?;


    if let Err(e) = image::save_buffer_with_format(output.name, &output.data, output.width, output.height, image::ColorType::Rgba8, image_format_1) {
        Err(UnableToSaveImage(e))
    } else { Ok(()) }
}

fn find_image_from_path(path: String) -> Result<(DynamicImage, ImageFormat), ImageDataErros> {
    match Reader::open(&path) {
        Ok(image_reader) => {
            if let Some(image_format) = image_reader.format() {
                match image_reader.decode() {
                    Ok(image) => Ok((image, image_format)),
                    Err(e) => Err(ImageDataErros::UnableToDecodeImage(e))
                }
            } else {
                return Err(ImageDataErros::UnableToFormatImage(path))
            }
        },
        Err(e) => Err(ImageDataErros::UnableToReadImageFromPath(e))
    }
}

fn get_smallest_dimension(dim_1: (u32, u32), dim_2: (u32, u32)) -> (u32, u32) {
    let pix_1 = dim_1.0 * dim_1.1;
    let pix_2 = dim_2.0 * dim_2.1;
    return if pix_1 < pix_2 { dim_1 } else { dim_2 }
}

fn standardise_size(image_1: DynamicImage, image_2: DynamicImage) -> (DynamicImage, DynamicImage) {
    let (width, height) = get_smallest_dimension(image_1.dimensions(), image_2.dimensions());
    println!("width: {}, height: {}\n", width, height);

    if image_2.dimensions() == (width, height) {
        (image_1.resize_exact(width, height, FilterType::Triangle), image_2)
    } else {
        (image_2.resize_exact(width, height, FilterType::Triangle), image_1)
    }
}

fn combine_images(image_1: DynamicImage, image_2: DynamicImage) -> Vec<u8> {
    let vec_1 = image_1.to_rgba8().into_vec();
    let vec_2 = image_2.to_rgba8().into_vec();

    alternate_pixels(vec_1, vec_2)
}

fn alternate_pixels(vec_1: Vec<u8>, vec_2: Vec<u8>) -> Vec<u8> {
    //If vec_1.len() == 5, [0, 0, 0, 0, 0]
    let mut combined_data = vec![0u8; vec_1.len()];

    let mut i = 0;
    while i < vec_1.len() {
        if i % 8 == 0 {
            combined_data.splice(i..=i + 3, set_rgba(&vec_1, i, i + 3));
        } else {
            combined_data.splice(i..=i + 3, set_rgba(&vec_2, i, i + 3)); }
        i += 4;
    }
    combined_data
}

fn set_rgba(vec: &Vec<u8>, start: usize, end: usize) -> Vec<u8> {
    let mut rgba = Vec::new();
    for i in start..=end {
        let val:u8 = match vec.get(i) {
            Some(d) => *d,
            None => panic!("Out of bounds")
        };
        rgba.push(val);
    }
    rgba
}