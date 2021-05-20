extern crate ft_by_hand;

use std::env;

use ft_by_hand::*;

fn get_user_input() -> DataT {
    let args: Vec<String> = env::args().collect();
    assert!(args.len() > 1, "no values passed in program arguments");
    return args
        .iter()
        .skip(1)
        .map(|val| Complex::new(val.parse::<f64>().unwrap(), 0.0))
        .collect::<DataT>();
}

fn main() {
    let input = get_user_input();
    println!("Input: {:?}\n", input);
    let dft_output = calculate_dft(&input);
    let fft_output = calculate_fft(&input);
    println!("Output DFT:\n {:.3?}", dft_output);
    println!("Output FFT:\n {:.3?}", fft_output);
    println!("Output iDFT:\n {:.3?}", calculate_idft(&dft_output));
}
