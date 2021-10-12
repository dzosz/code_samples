fn find_repeated(arr : &[i32]) -> i32 {
    let ans = arr.iter().enumerate().fold(0, |acc, (idx, x)| acc ^ x ^ idx as i32);
    ans
}

fn main() {
    let arr = [1,2,3,4,2];

    let dupped = find_repeated(&arr);
    assert!(dupped == 2);
    println!("ok dupped is {}", dupped);
}
