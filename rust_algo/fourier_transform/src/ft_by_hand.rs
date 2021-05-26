extern crate num;

pub use num::Complex;

fn is_power_of_two(n: usize) -> bool {
    n != 0 && (n & (n - 1) == 0)
}

enum FftType {
    Normal,
    Inverse,
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
fn _calculate_fft(input: &mut DataT, t: &FftType) {
    let n = input.len();
    if n <= 1 {
        return;
    }

    // divide
    let mut even: DataT = input.iter().step_by(2).copied().collect();
    let mut odd = input.iter().skip(1).step_by(2).copied().collect();

    // conquer
    _calculate_fft(&mut even, &t);
    _calculate_fft(&mut odd, &t);

    let direction = match t {
        FftType::Inverse => 1.0,
        _ => -1.0,
    };
    // combine
    for k in 0..n / 2 {
        let t = polar(
            1.0,
            direction * 2.0 * k as f64 * std::f64::consts::PI / n as f64,
        ) * odd[k];
        input[k] = even[k] + t;
        input[k + n / 2] = even[k] - t;
    }
}

struct ComplexComparatorWrapper<'a>(pub &'a DataT);
impl PartialEq for ComplexComparatorWrapper<'_> {
    fn eq(&self, other: &Self) -> bool {
        let arbitrary_acceptable_difference = 0.001;
        self.0.len() == self.0.len()
            && self.0.iter().zip(other.0.iter()).all(|(left, right)| {
                num::abs(left.re - right.re) < arbitrary_acceptable_difference
                    && num::abs(left.im - right.im) < arbitrary_acceptable_difference
            })
    }
}

impl Eq for ComplexComparatorWrapper<'_> {}

pub type DataT = Vec<Complex<f64>>;

//  rearranges a Fourier transform X by shifting the zero-frequency component to the center of the array.
//  For vectors, fftshift(X) swaps the left and right halves of X.
//  For matrices, fftshift(X) swaps quadrants one and three of X with quadrants two and four.
//  For higher-dimensional arrays, fftshift(X) swaps "half-spaces" of X along each dimension.
pub fn fft_shift(input: &DataT, dim: usize) -> DataT {
    let mut output = input.clone();
    let size = input.len();
    let side = (size as f64).sqrt().round() as usize;

    match dim {
        1 |2 => {
            output.rotate_right(size/2);
            output.chunks_mut(side).for_each(|chunk| chunk.rotate_right(side/2));
            /*
            for src in 0..size {
                let dst = (src + (size / 2) - 1) % size;
                output.swap(src, dst);
                println!("swap {}->{}", src, dst);
            }*/
        }
        _ => {
            panic!("fft higher shift dimension={} are not handled", dim);
        }
    }
    output
}

pub fn ifft_shift(input: &DataT, dim: usize) -> DataT {
    let mut output = input.clone();
    let size = input.len();
    let side = (size as f64).sqrt().round() as usize;

    match dim {
        1 |2 => {
            output.rotate_left(size/2);
            output.chunks_mut(side).for_each(|chunk| chunk.rotate_left(side/2));
        }
        _ => {
            panic!("fft higher shift dimension={} are not handled", dim);
        }
    }
    output
}

pub fn calculate_dft(input: &DataT) -> DataT {
    let mut output: DataT = Vec::new();
    output.resize_with(input.len(), Default::default);

    for (k, elem) in output.iter_mut().enumerate() {
        for j in 0..input.len() {
            *elem += polar(
                1.0,
                -2.0 * (k * j) as f64 * std::f64::consts::PI / input.len() as f64,
            ) * input[j].re;
        }
    }
    output
}

pub fn calculate_idft(input: &DataT) -> DataT {
    let mut output: DataT = Vec::new();
    output.resize_with(input.len(), Default::default);

    for (k, elem) in output.iter_mut().enumerate() {
        for j in 0..input.len() {
            *elem += polar(
                1.0,
                2.0 * (k * j) as f64 * std::f64::consts::PI / input.len() as f64,
            ) * input[j];
        }
        *elem = elem.unscale(input.len() as f64);
    }
    output
}

fn transpose(input: &DataT) -> DataT {
    let mut output = input.clone();
    let side = (input.len() as f64).sqrt().round() as usize;
    for i in 0..side {
        for j in i..side {
            let left = i + j*side;
            let right = i*side + j;
            output.swap(left, right);
        }
    }
    output
}

pub fn calculate_fft(input: &DataT) -> DataT {
    assert!(
        is_power_of_two(input.len()),
        "this fft algorithm requires input size of power of 2"
    );
    let mut output = input.clone();

    _calculate_fft(&mut output, &FftType::Normal);
    output
}

// TODO there is better algorithm for that
pub fn calculate_fft2(input: &DataT) -> DataT {
    assert!(
        is_power_of_two(input.len()),
        "this fft algorithm requires input size of power of 2"
    );
    let mut output = input.clone();

    let side = (input.len() as f64).sqrt().round() as usize;
    for i in (0..input.len()).step_by(side) {
        let mut tmp = output[i..i+side].to_vec();
        _calculate_fft(&mut tmp, &FftType::Normal);
        output.splice(i..i+side, tmp);
    }
    output = transpose(&mut output);
    for i in (0..input.len()).step_by(side) {
        let mut tmp = output[i..i+side].to_vec();
        _calculate_fft(&mut tmp, &FftType::Normal);
        output.splice(i..i+side, tmp);
    }
    output = transpose(&mut output);

    output
}

pub fn calculate_ifft(input: &DataT) -> DataT {
    assert!(
        is_power_of_two(input.len()),
        "this fft algorithm requires input size of power of 2"
    );
    let mut output = input.clone();

    _calculate_fft(&mut output, &FftType::Inverse);
    for elem in output.iter_mut() {
        *elem = elem.unscale(input.len() as f64);
    }
    output
}

pub fn calculate_ifft2(input: &DataT) -> DataT {
    assert!(
        is_power_of_two(input.len()),
        "this fft algorithm requires input size of power of 2"
    );
    let mut output = input.clone();

    // by row
    let side = (input.len() as f64).sqrt().round() as usize;
    for i in (0..input.len()).step_by(side) {
        let mut tmp = output[i..i+side].to_vec();
        _calculate_fft(&mut tmp, &FftType::Inverse);
        output.splice(i..i+side, tmp);
    }
    output = transpose(&mut output);
    // by column
    for i in (0..input.len()).step_by(side) {
        let mut tmp = output[i..i+side].to_vec();
        _calculate_fft(&mut tmp, &FftType::Inverse);
        output.splice(i..i+side, tmp);
    }
    for elem in output.iter_mut() {
        *elem = elem.unscale(input.len() as f64);
    }
    output = transpose(&mut output);

    output
}

pub fn eq_complex_vector(left: &DataT, right: &DataT) -> bool {
    ComplexComparatorWrapper(&left) == ComplexComparatorWrapper(&right)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generate_n_complex_values(n: usize) -> DataT {
        (1..n + 1).map(|v| Complex::new(v as f64, 0.0)).collect()
    }

    #[test]
    fn dft_and_fft_outputs_same_values() {
        let input = generate_n_complex_values(4);
        let dft_output = calculate_dft(&input);
        let fft_output = calculate_fft(&input);
        let equal_dft_fft = eq_complex_vector(&dft_output, &fft_output);
        assert!(equal_dft_fft, "computations from FFT and DFT are not equal")
    }

    #[test]
    fn idft_and_ifft_outputs_same_values() {
        let input = generate_n_complex_values(4);
        let idft_output = calculate_idft(&input);
        let ifft_output = calculate_ifft(&input);
        let equal_dft_fft = eq_complex_vector(&idft_output, &ifft_output);
        assert!(
            equal_dft_fft,
            "computations from iFFT and iDFT are not equal"
        )
    }

    #[test]
    fn fft_shift_simple_vector() {
        let input = generate_n_complex_values(4);
        let expected = vec![3, 4, 1, 2]
            .into_iter()
            .map(|val| Complex::new(val as f64, 0.))
            .collect::<DataT>();
        let result = fft_shift(&input, 1);
        assert!(
            eq_complex_vector(&result, &expected),
            "Not equal!\nexp={:?}\nres={:?}",
            expected,
            result
        )
    }

    #[test]
    fn fft_shift_simple_matrix() {
        let input = generate_n_complex_values(4);
        let expected = vec![3, 4, 1, 2]
            .into_iter()
            .map(|val| Complex::new(val as f64, 0.))
            .collect::<DataT>();
        let result = fft_shift(&input, 2);
        assert!(
            eq_complex_vector(&result, &expected),
            "Not equal!\nexp={:?}\nres={:?}",
            expected,
            result
        )
    }

    #[test]
    fn ifft_shift_simple_vector() {
        let input = generate_n_complex_values(4);
        let result = ifft_shift(&fft_shift(&input, 1),1);
        assert!(
            eq_complex_vector(&result, &input),
            "Not equal!\nexp={:?}\nres={:?}",
            input,
            result
        )
    }

    #[test]
    fn ifft_shift_simple_matrix() {
        let input = generate_n_complex_values(4);
        let result = ifft_shift(&fft_shift(&input, 2),2);
        assert!(
            eq_complex_vector(&result, &input),
            "Not equal!\nexp={:?}\nres={:?}",
            input,
            result
        )
    }
    #[test]
    fn transpose_matrix() {
        let input = generate_n_complex_values(4);
        let expected = vec![1, 3, 2, 4]
            .into_iter()
            .map(|val| Complex::new(val as f64, 0.))
            .collect::<DataT>();
        let result = transpose(&input);
        assert!(
            eq_complex_vector(&result, &expected),
            "transpose failed\n{:?}\n{:?}", result, expected
        )
    }
    #[test]
    fn fft2_matrix() {
        let input = generate_n_complex_values(4);
        let expected = vec![10, -2, -4, 0]
            .into_iter()
            .map(|val| Complex::new(val as f64, 0.))
            .collect::<DataT>();
        let result = calculate_fft2(&input);
        assert!(
            eq_complex_vector(&result, &expected),
            "fft2 matrix failed\nres={:?}\nexp={:?}", result,expected
        )
    }
    #[test]
    fn ifft2_matrix() {
        let input = generate_n_complex_values(4);
        let result = calculate_ifft2(&calculate_fft2(&input));
        assert!(
            eq_complex_vector(&result, &input),
            "fft2 matrix failed\nres={:?}\nexp={:?}", result, input
        )
    }
}
