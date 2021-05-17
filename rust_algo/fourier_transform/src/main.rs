extern crate num;

extern crate float_cmp;


use float_cmp::approx_eq;

use num::Complex;
use std::env;

type InputT = Vec<f64>;
type OutputT = Vec<Complex<f64>>;


// The sine and cosine waves are called DFT basic functions - they are waves with unity amplitude. The DFT basic functions have the following equations:
// By eulers formula e^(i*theta) = cos(theta) + i * sin(theta)
// ck[i] = cos(2pi * k * i/N)
// sk[i] = sin(2pi * k * i/N)
// x[i]  = sum(ck) + i * sum(sk)

fn transform_using_waves(k : usize, j : usize, n : usize) -> Complex<f64>{
    let common = (2*k * j) as f64 * std::f64::consts::PI * -1.0 / n as f64;
    Complex::new(common.cos(), common.sin())
}

fn calculate_dft(input: &InputT) -> OutputT {
    let mut output: OutputT = Vec::new();
    output.resize_with(input.len(), Default::default);

    for (k, elem) in output.iter_mut().enumerate() {
        for j in 0..input.len() {
            *elem += transform_using_waves(k, j, input.len()) * input[j] as f64;
        }
    }
    output
}

fn calculate_fft(input: &InputT) -> OutputT {
    let mut output: OutputT = input.iter().map(|val| Complex::new(*val, 0.0)).collect::<OutputT>();
    _calculate_fft(&mut output);
    output
}

// Cooley–Tukey FFT (in-place, divide-and-conquer)
// Higher memory requirements and redundancy although more intuitive
// adapted from c++ code from https://rosettacode.org/wiki/Fast_Fourier_transform
fn _calculate_fft(input: &mut OutputT) {
    let n = input.len();
    if n <= 1 { return }
    assert!(n %2 == 0);

    // divide
    let mut even : OutputT = input.iter().step_by(2).copied().collect();
    let mut odd = input.iter().skip(1).step_by(2).copied().collect();

    // conquer
    _calculate_fft(&mut even);
    _calculate_fft(&mut odd);

    // combine
    for k in 0..n/2 {
        // t= std::complex(r * cos(theta), r * sin(theta))
        let t = transform_using_waves(k, 1, n) * odd[k];
        input[k    ] = even[k] + t;
        input[k+n/2] = even[k] - t;
    }


}

fn get_optional_user_input() -> InputT {
    let args : Vec<String> = env::args().collect();
    if args.len() > 1 {
        return args.iter().skip(1).map(|val| val.parse::<f64>().unwrap()).collect::<Vec<f64>>();
    }
    return vec![1.0,2.0,3.0,4.0];
}

//struct ComplexComparatorWrapper(pub OutputT);
struct ComplexComparatorWrapper(pub OutputT);
impl PartialEq for ComplexComparatorWrapper {
    fn eq(&self, other: &Self) -> bool {
        self.0.len() == self.0.len() && self.0.iter().zip(other.0.iter()).any(|(left, right)| !approx_eq!(f64, left.re, right.re, ulps=2) || !approx_eq!(f64, left.im, right.im, ulps=2) )
    }
}
impl Eq for ComplexComparatorWrapper {}


fn main() {
    let input = get_optional_user_input();
    println!("Input: {:?}\n", input);
    let dft_output = calculate_dft(&input);
    let fft_output = calculate_fft(&input);
    println!("Output DFT:\n {:?}", dft_output);
    println!("Output FFT:\n {:?}", fft_output);
    let equal = ComplexComparatorWrapper(dft_output) == ComplexComparatorWrapper(fft_output);// TODO use refernce wrapper?
    println!("\nIs (FFT==DFT) {}", equal);
    assert!(equal, "computations from FFT and DFT are not equal");
}
