type Matrix = Vec<Vec<i32>>;

const TEST_DATA: [[i32; 5]; 5] = [
    [0, 2, 0, 6, 0],
    [2, 0, 3, 8, 5],
    [0, 3, 0, 0, 7],
    [6, 8, 0, 0, 9],
    [0, 5, 7, 9, 0],
];

struct Graph {
    data: Matrix,
}


impl Graph {
    fn new(data: Matrix) -> Self {
        Self { data: data }
    }

    fn findNextVertex(&self, weights: &Vec<i32>, visited: &Vec<bool>) -> usize {
        let mut min_index: i32 = -1;
        let mut best_weight = std::i32::MAX;
        for v in 0..self.data.len() {
            if weights[v] < best_weight && !visited[v] {
                min_index = v as i32;
                best_weight = weights[v];
            }
        }
        if min_index == -1 {
            panic!("could not find index");
        }
        min_index as usize
    }

    fn createConnectionMatrix(&self, mst : Vec<i32>) -> Matrix {
        let mut connection_matrix = self.data.clone();
        connection_matrix.iter_mut().for_each(|vec| vec.iter_mut().for_each(|x| *x=0));

        for idx1 in 0..mst.len() {
            let idx2 = mst[idx1] as usize;
            connection_matrix[idx1][idx2] = self.data[idx1][idx2];
            connection_matrix[idx2][idx1] = self.data[idx1][idx2];
        }
            

        connection_matrix
    }

    fn calculateMST(&self) -> Matrix {
        let elems = self.data.len();
        let mut weights = vec![std::i32::MAX; elems];
        let mut mst = vec![0; elems];

        weights[0] = 0;
        let mut visited = vec![false; elems];

        for _ in 0..elems {
            let first_vertex = self.findNextVertex(&weights, & visited);
            visited[first_vertex] = true;

            for second_vertex in 0..elems {
                if self.data[first_vertex][second_vertex] > 0
                    && !visited[second_vertex]
                    && weights[second_vertex] > self.data[first_vertex][second_vertex]
                {
                    weights[second_vertex] = self.data[first_vertex][second_vertex];
                    mst[second_vertex] = first_vertex as i32;
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
}
