extern crate ft_by_hand;
extern crate nalgebra as na;
extern crate num;
extern crate png;

pub use num::Complex;
use std::env;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use ft_by_hand::*;

fn get_user_filename() -> String {
    let args: Vec<String> = env::args().collect();
    assert!(
        args.len() == 2,
        "no path to png file passed in program arguments"
    );
    args[1].parse::<String>().unwrap()
}

fn write_file(fname: String, data: &[u8], width: u32, height: u32) {
    println!("writing file {}", fname);

    let path = Path::new(&fname);
    let file = File::create(path).unwrap();
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, height, width);
    encoder.set_color(png::ColorType::Grayscale);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();

    writer.write_image_data(&data).unwrap(); // Save
}

fn rgba_to_grayscale(img_src: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();
    for i in (0..img_src.len()).step_by(4) {
        let rgba_row = &img_src[i..i + 4];
        let luminosity =
            0.299 * rgba_row[0] as f64 + 0.587 * rgba_row[1] as f64 + 0.114 * rgba_row[2] as f64;
        result.push(luminosity as u8);
    }
    result
}

#[allow(dead_code)]
fn transform_magnitude_spectrum(input: &DataT) -> DataT {
    input
        .iter()
        .map(|&val| {
            Complex::new(
                20. * ((val.im * val.im) + (val.re + val.re)).sqrt().ln(),
                0.,
            )
        })
        .collect::<DataT>()
}

fn blur_image(img_src: &DataT) -> DataT {
    let side = (img_src.len() as f64).sqrt().round() as usize;

    let mask_size = side;
    let mut mask =
        na::DMatrix::<Complex<f64>>::from_element(mask_size, mask_size, Default::default());

    let blur_size = mask_size / 5;
    let start = (mask_size / 2) - (blur_size / 2);
    for i in 0..blur_size {
        for j in 0..blur_size {
            mask[(i + start, j + start)] = Complex::new(1.0, 1.0);
        }
    }

    // some implementations say we should fft mask first
    /*
    unsafe {
        *mask.data.as_vec_mut() = calculate_fft2(&mask.data.as_vec().clone());
    }

    for i in 0..blur_size {
        for j in 0..blur_size {
            if mask[(i+start,j+start)].im == 0. {
                mask[(i+start,j+start)].im = 1e-6;
            }
            if mask[(i+start,j+start)].re == 0. {
                mask[(i+start,j+start)].re = 1e-6;
            }
        }
    }*/

    let img = na::Matrix::from_vec_generic(
        na::Dynamic::new(side),
        na::Dynamic::new(side),
        img_src.clone(),
    );
    let blurred = img.component_mul(&mask);

    blurred.data.as_vec().clone()
}

fn main() {
    let fname = get_user_filename();
    let decoder = png::Decoder::new(File::open(fname.clone()).unwrap());
    let (info, mut reader) = decoder.read_info().unwrap();
    let mut buf = vec![0; info.buffer_size()];
    reader.next_frame(&mut buf).unwrap();

    let (width, height) = (info.height, info.width);

    println!(
        "image w={} h={} colortype={:?}",
        width, height, info.color_type
    );
    assert!(
        info.color_type == png::ColorType::RGBA,
        "image must be RGBA"
    );

    let gray_image_complex: DataT = rgba_to_grayscale(&buf)
        .iter()
        .map(|val| Complex::new(*val as f64, 0.0))
        .collect();

    let blurred_gray_image = calculate_ifft2(&ifft_shift(
        &blur_image(&fft_shift(&calculate_fft2(&gray_image_complex), 2)),
        2,
    ))
    .iter()
    .map(|val| val.re as u8)
    .collect::<Vec<u8>>();

    write_file(fname.clone() + "2", &blurred_gray_image[..], width, height);
}
