type Matrix = Vec<Vec<i32>>;
type Tree = std::collections::HashSet<i32>;
type Trees = Vec<Tree>;
type DataT = [[i32; 5]; 5];

use std::convert::TryInto;

const TEST_DATA: DataT = [
    [0, 2, 0, 6, 0],
    [2, 0, 3, 8, 5],
    [0, 3, 0, 0, 7],
    [6, 8, 0, 0, 9],
    [0, 5, 7, 9, 0],
];

const TEST_DATA_SOLVED: DataT = [
    [0, 2, 0, 6, 0],
    [2, 0, 3, 0, 5],
    [0, 3, 0, 0, 0],
    [6, 0, 0, 0, 0],
    [0, 5, 0, 0, 0],
];

fn vec_to_arr(data: &Matrix) -> DataT {
    let mut arr: DataT = Default::default();
    for idx1 in 0..data.len() {
        for idx2 in 0..data.len() {
            arr[idx1][idx2] = data[idx1][idx2];
            arr[idx2][idx1] = data[idx1][idx2];
        }
    }
    arr
}

struct Graph {
    data: Matrix,
}

impl Graph {
    fn new(data: Matrix) -> Self {
        Self { data: data }
    }

    fn createConnectionMatrix(&self, mst: Vec<i32>) -> Matrix {
        let mut connection_matrix = self.data.clone();
        connection_matrix
            .iter_mut()
            .for_each(|vec| vec.iter_mut().for_each(|x| *x = 0));

        for idx1 in 0..mst.len() {
            let idx2 = mst[idx1] as usize;
            connection_matrix[idx1][idx2] = self.data[idx1][idx2];
            connection_matrix[idx2][idx1] = self.data[idx1][idx2];
        }

        connection_matrix
    }

    fn isFinished(&self, mst : &Vec<i32>) -> bool {
        for v2 in (1..self.data.len()) {
            let mut nextVert = mst[v2];
            let mut visited = vec![false; self.data.len()];
            let ok = self.reaches_first_node(mst, v2, &mut visited);
            if !ok {
                return false;
            }
        }
        return true;
    }

    fn reaches_first_node(&self, mst :&Vec<i32>, vert : usize, visited : &mut Vec<bool>) -> bool {
        if vert == 0 {
            return true;
        }
        if !visited[vert] {
            visited[vert] = true;
            return self.reaches_first_node(mst, mst[vert].try_into().unwrap(), visited);
        }
        return false;

    }

    fn findNextMinimumEdge(&self, v1 : usize, visited : &Vec<Vec<bool>>) -> usize {
        let mut min_index: i32 = -1;
        let mut best_weight = std::i32::MAX;
        for v2 in 0..self.data.len() {
            if self.data[v1][v2] < best_weight && !visited[v1][v2] && self.data[v1][v2] > 0{
                min_index = v2 as i32;
                best_weight = self.data[v1][v2];
            }
        }
        if min_index == -1 {
            panic!("no minimum edge");
        }
        min_index as usize
    }

    fn calculateMST(&self) -> Matrix {
        let elems = self.data.len();
        let mut mst = vec![0; elems];

        let mut visited = vec![vec![false; elems]; elems];

        loop {
            for v in 0..self.data.len() {
                let v2 = self.findNextMinimumEdge(v, &visited);
                visited[v][v2] = true;
                //visited[v2][v] = true;
                // println!("min {} {}={}", v, v2, self.data[v][v2]);
                if mst[v as usize] == 0 {
                    mst[v as usize] = v2 as i32;
                } else {
                    mst[v2 as usize] = v as i32;
                }
            }
            //println!("{:?} {}\n", mst, self.isFinished(&mst));
            if self.isFinished(&mst)  {
                break;
            }
        }
        self.createConnectionMatrix(mst)
    }
}

fn main() {
    let vectored = TEST_DATA.iter().map(|arr| arr.to_vec()).collect();
    println!(" in: {:?}", vectored);

    let g = Graph::new(vectored);
    let result_matrix = g.calculateMST();
    println!("out: {:?}", result_matrix);

    let arrayed = vec_to_arr(&result_matrix);
    assert_eq!(TEST_DATA_SOLVED, arrayed);
}
