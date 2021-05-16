extern crate num;

use num::Complex;

const LEN: usize = 4;

type OutputT = [Complex<f64>; LEN];


// The sine and cosine waves are called DFT basic functions - they are waves with unity amplitude. The DFT basic functions have the following equations:
// ck[i] = cos(2pi * k * i/N)
// sk[i] = sin(2pi * k * i/N)
// x[i]  = sum(ck) + i * sum(sk)

fn transform_using_waves(k : usize, j : usize) -> Complex<f64>{
    let common = (2*k * j) as f64 * std::f64::consts::PI * -1.0 / LEN as f64;
    Complex::new(common.cos(), common.sin())
}

fn calculate_dft(input: [i32; LEN]) -> OutputT {
    let mut output: OutputT = Default::default();

    for (k, elem) in output.iter_mut().enumerate() {
        for j in 0..LEN {
            *elem += transform_using_waves(k, j) * input[j] as f64;
        }
        println!("fk^ {} : {}", k, elem);
    }
    output
}

fn main() {
    let input: [i32; LEN] = [1, 2, 3, 4];
    println!("Input: {:?}", input);
    let output = calculate_dft(input);
    println!("Output: {:?}", output);
}
