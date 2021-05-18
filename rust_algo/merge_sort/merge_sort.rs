type Cont = Vec<i32>;
type Arr = [i32];

fn merge_sort(nums : &mut Arr) {
    let n = nums.len();
    if n < 2 {
        return;
    }

    // divide
    let (mut left, mut right)  = (|| {
        let (l, r) = nums.split_at(n/2);
        (l.to_vec(), r.to_vec())
    })();

    // conquer
    merge_sort(&mut left.as_mut_slice());
    merge_sort(&mut right.as_mut_slice());

    let (mut leftidx, mut rightidx) = (0,0);


    // combine
    for i in 0..n {
        let smaller;
        if leftidx < left.len() && (rightidx >= right.len() || left[leftidx] <= right[rightidx]) {
            smaller = left[leftidx];
            leftidx += 1;
        } else {
            smaller = right[rightidx];
            rightidx += 1;
        }

        nums[i] = smaller;
    }
}

fn get_optional_user_input() -> Cont {
    let args: Vec<String> = std::env::args().collect();
    println!("args: {:?}", args);
    if args.len() > 1 {
        return args
            .iter()
            .skip(1)
            .map(|val| val.parse::<i32>().unwrap())
            .collect::<Vec<i32>>();
    }
    return vec![10, 2, 30, 4];
}

fn main() {
    let mut input = get_optional_user_input();
    println!("Input: {:?}\n", input);
    merge_sort(input.as_mut_slice());
    println!("Output merge sort:\n {:?}", input);
}
