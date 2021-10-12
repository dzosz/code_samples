// input must be consecutive numbers except for the missing
fn find_missing(arr : &[i32]) -> i32 {
    let nums = arr.iter().fold(0, |acc, x| acc ^ x);
    let all_nums = (1..arr.len()+2).fold(0, |acc, x| acc ^ x); // xor all the numbers in range
    nums ^ all_nums as i32
}

fn main() {
    let arr = [1,2,3,4 /*missing 5*/ , 6];
    let missing = find_missing(&arr);
    println!("ok missing is {}", missing);
    assert!(missing == 5);
}
