type Matrix = Vec<Vec<i32>>;
type Tree = std::collections::HashSet<i32>;
type Trees = Vec<Tree>;
type DataT = [[i32; 5]; 5];

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

    fn calculateMST(&self) -> Matrix {
        let elems = self.data.len();
        let mut mst = vec![0; elems];

        let mut sub_trees: Trees = Default::default(); // TODO use smarter data structure?
        (0..elems as i32).for_each(|elem| {
            let mut t = Tree::new();
            t.insert(elem);
            sub_trees.push(t);
        });

        let sorted_edges = {
            let mut edges: Vec<(i32, i32)> = Default::default();
            for idx1 in 0..mst.len() - 1 {
                for idx2 in idx1 + 1..mst.len() {
                    if self.data[idx1][idx2] > 0 {
                        edges.push((idx1 as i32, idx2 as i32));
                    }
                }
            }
            edges.sort_by(|(a, b), (c, d)| {
                self.data[*a as usize][*b as usize]
                    .partial_cmp(&self.data[*c as usize][*d as usize])
                    .unwrap()
            });
            edges
        };

        for (vert1, vert2) in sorted_edges {
            let intersection = sub_trees[vert1 as usize].intersection(&sub_trees[vert2 as usize]);
            let is_not_a_cycle = intersection.count() < 2;
            if is_not_a_cycle {
                if mst[vert1 as usize] == 0 {
                    mst[vert1 as usize] = vert2;
                } else {
                    mst[vert2 as usize] = vert1;
                }

                // TODO we can do it without allocating new hashset using vector.split_at_mut()
                {
                    let un: std::collections::HashSet<_> = sub_trees[vert1 as usize]
                        .union(&sub_trees[vert2 as usize])
                        .cloned()
                        .collect();
                    sub_trees[vert1 as usize].extend(un);
                }
                {
                    let un: std::collections::HashSet<_> = sub_trees[vert2 as usize]
                        .union(&sub_trees[vert1 as usize])
                        .cloned()
                        .collect();
                    sub_trees[vert2 as usize].extend(un);
                }
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
