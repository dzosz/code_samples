extern crate num;

use num::Complex;
use std::env;

type DataT = Vec<Complex<f64>>;

fn is_power_of_two(n : usize) -> bool {
    n != 0 && (n & (n-1) == 0)
}

fn calculate_dft(input: &DataT) -> DataT {
    let mut output: DataT = Vec::new();
    output.resize_with(input.len(), Default::default);

    for (k, elem) in output.iter_mut().enumerate() {
        for j in 0..input.len() {
            *elem += polar(1.0, -2.0 * (k*j) as f64 * std::f64::consts::PI / input.len() as f64) * input[j].re;
        }
    }
    output
}

fn calculate_idft(input: &DataT) -> DataT {
    let mut output: DataT = Vec::new();
    output.resize_with(input.len(), Default::default);

    for (k, elem) in output.iter_mut().enumerate() {
        for j in 0..input.len() {
            *elem += polar(1.0, 2.0 * (k*j) as f64 * std::f64::consts::PI / input.len() as f64) * input[j];
        }
        *elem = elem.unscale(input.len() as f64);
    }
    output
}

fn calculate_fft(input: &DataT) -> DataT {
    assert!(is_power_of_two(input.len()), "this fft algorithm requires input size of power of 2");
    let mut output = input.clone();

    _calculate_fft(&mut output);
    output
}


// The sine and cosine waves are called DFT basic functions - they are waves with unity amplitude. The DFT basic functions have the following equations:
// By eulers formula e^(i*theta) = cos(theta) + i * sin(theta)
// ck[i] = cos(2pi * k * i/N)
// sk[i] = sin(2pi * k * i/N)
// x[i]  = sum(ck) + i * sum(sk)
// Returns a complex number with magnitude r and phase angle theta.
fn polar(magnitude: f64, phase_angle: f64) -> Complex<f64> {
    Complex::new(phase_angle.cos(), phase_angle.sin()).scale(magnitude)
}

// Cooley–Tukey FFT (in-place, divide-and-conquer)
// Higher memory requirements and redundancy although more intuitive
// adapted from c++ code from https://rosettacode.org/wiki/Fast_Fourier_transform
fn _calculate_fft(input: &mut DataT) {
    let n = input.len();
    if n <= 1 {
        return;
    }

    // divide
    let mut even: DataT = input.iter().step_by(2).copied().collect();
    let mut odd = input.iter().skip(1).step_by(2).copied().collect();

    // conquer
    _calculate_fft(&mut even);
    _calculate_fft(&mut odd);

    // combine
    for k in 0..n / 2 {
        let t = polar(1.0, -2.0 * k as f64 * std::f64::consts::PI / n as f64) * odd[k];
        input[k] = even[k] + t;
        input[k + n / 2] = even[k] - t;

    }
}

fn get_optional_user_input() -> DataT {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        return args
            .iter()
            .skip(1)
            .map(|val| Complex::new(val.parse::<f64>().unwrap(), 0.0))
            .collect::<DataT>();
    }
    return vec![Complex{re:1.0,im:0.0}, Complex{re:2.0, im:0.0}, Complex{re:3.0, im:0.0}, Complex{re:4.0,im:0.0}]; // default
}

struct ComplexComparatorWrapper<'a>(pub &'a DataT);
impl PartialEq for ComplexComparatorWrapper<'_> {
    fn eq(&self, other: &Self) -> bool {
        let arbitraryAcceptableDifference = 0.001;
        self.0.len() == self.0.len()
            && self.0.iter().zip(other.0.iter()).all(|(left, right)| {
                num::abs(left.re - right.re) < arbitraryAcceptableDifference
                && num::abs(left.im - right.im) < arbitraryAcceptableDifference
            })
    }
}
impl Eq for ComplexComparatorWrapper<'_> {}

fn main() {
    let input = get_optional_user_input();
    println!("Input: {:?}\n", input);
    let dft_output = calculate_dft(&input);
    let fft_output = calculate_fft(&input);
    println!("Output DFT:\n {:.3?}", dft_output);
    println!("Output FFT:\n {:.3?}", fft_output);
    println!("Output iDFT:\n {:.3?}", calculate_idft(&dft_output));

    let equal_DFT_FFT = ComplexComparatorWrapper(&dft_output) == ComplexComparatorWrapper(&fft_output);
    println!("\nIs (FFT==DFT) {}", equal_DFT_FFT);
    assert!(equal_DFT_FFT, "computations from FFT and DFT are not equal");
}
