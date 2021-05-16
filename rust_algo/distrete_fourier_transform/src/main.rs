extern crate num;

use num::Complex;
use std::env;

type InputT = Vec<i32>;
type OutputT = Vec<Complex<f64>>;


// The sine and cosine waves are called DFT basic functions - they are waves with unity amplitude. The DFT basic functions have the following equations:
// ck[i] = cos(2pi * k * i/N)
// sk[i] = sin(2pi * k * i/N)
// x[i]  = sum(ck) + i * sum(sk)

fn transform_using_waves(k : usize, j : usize, n : usize) -> Complex<f64>{
    let common = (2*k * j) as f64 * std::f64::consts::PI * -1.0 / n as f64;
    Complex::new(common.cos(), common.sin())
}

fn calculate_dft(input: InputT) -> OutputT {
    let mut output: OutputT = Vec::new();
    output.resize_with(input.len(), Default::default);

    for (k, elem) in output.iter_mut().enumerate() {
        for j in 0..input.len() {
            *elem += transform_using_waves(k, j, input.len()) * input[j] as f64;
        }
    }
    output
}

fn get_optional_user_input() -> InputT {
    let args : Vec<String> = env::args().collect();
    if args.len() > 1 {
        return args.iter().skip(1).map(|val| val.parse::<i32>().unwrap()).collect::<Vec<i32>>();
    }
    return vec![1,2,3,4];
}

fn main() {
    let input = get_optional_user_input();
    println!("Input: {:?}", input);
    let output = calculate_dft(input);
    println!("Output: {:?}", output);
}
