use std::thread;
use std::fmt;
use tokio::sync::broadcast;
use tokio::sync::mpsc;

type Verts = Vec<usize>;
type Matrix = Vec<Verts>;
type DataT = [[usize; 5]; 5];
type Data2 = [[usize; 3]; 3];

pub const TEST_DATA: DataT = [
    [0, 2, 0, 6, 0],
    [2, 0, 3, 8, 5],
    [0, 3, 0, 0, 7],
    [6, 8, 0, 0, 9],
    [0, 5, 7, 9, 0],
];

pub const TEST_DATA2: Data2 = [
    [0, 1, 3],
    [1, 0, 2],
    [3, 2, 0]
];

const TEST_DATA_SOLVED: DataT = [
    [0, 2, 0, 6, 0],
    [2, 0, 3, 0, 5],
    [0, 3, 0, 0, 0],
    [6, 0, 0, 0, 0],
    [0, 5, 0, 0, 0],
];

#[derive(Copy, Clone, PartialEq, Debug)]
enum EdgeState {
    Basic,
    Requested,
    Branch,
    Rejected,
}

impl Default for EdgeState {
    fn default() -> EdgeState {
        EdgeState::Basic
    }
}

#[derive(Clone)]
struct Node {
    id: usize,
    vertices: Verts,
    states: Vec<EdgeState>,
    name: String,
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}){}", self.id, self.name)
    }
}

impl Node {
    fn new(id: usize, verts: Verts) -> Self {
        let len = verts.len();
        Self {
            id: id,
            vertices: verts,
            states: vec![Default::default(); len],
            name: String::from(""),
        }
    }

    fn has_pending(&self) -> bool {
        self.states.iter().any(|state| match state {
            EdgeState::Requested => true,
            _ => false,
        })
    }

    fn find_min_edge(&self) -> usize {
        let mut best = usize::MAX;
        let mut idx = None;
        for i in 0..self.vertices.len() {
            if i == self.id || self.vertices[i] == 0 {
                continue;
            }
            match self.states[i] {
                EdgeState::Basic => {
                    if self.vertices[i] < best { 
                        best = best.min(self.vertices[i]);
                        idx = Some(i);
                    }
                }
                _ => {}
            }
        }
        idx.unwrap()
    }

    // GHS terminates when all edges are in Branch/Rejected state
    fn is_done(&self) -> bool {
        self.states.iter().enumerate().all(|(i, state)| match state {
            EdgeState::Branch => true,
            EdgeState::Rejected => true,
            EdgeState::Requested => false,
            EdgeState::Basic  => i == self.id || self.vertices[i] == 0 ,
        })
    }
}

#[derive(Copy, Clone)]
enum RequestType {
    Search,
    Absorb,
    Reject,
}

#[derive(Copy, Clone)]
struct MessageRequest {
    to: usize,
    from: usize,
    level: usize,
    req: RequestType,
}

pub struct Graph {
    data: Matrix,
    nodes: Vec<Node>,
}

impl Graph {
    pub fn new(data: Matrix) -> Self {
        Self {
            data: data,
            nodes: Default::default(),
        }
    }

    fn init(&mut self) {
        for i in 0..self.data.len() {
            self.nodes.push(Node::new(i, self.data[i].clone()));
        }
    }

    pub async fn start(&mut self) {
        self.init();
        //println!("start {}", self.nodes.len());
        let (tx, mut rx1) = broadcast::channel(self.nodes.len() * 2);

        let mut handles: Vec<_> = Default::default();
        let mut tx_rx: Vec<_> = Default::default();

        // channels must be created earlier so all the messages are delivered
        for i in 0..self.data.len() {
            let tx2 = tx.clone();
            let mut rx = tx.subscribe();
            tx_rx.push((tx2, rx));
        }

        for i in 0..self.data.len() {
            //println!("start {}", i);
            let mut node = self.nodes[i].clone();
            //node.states[node.id] = EdgeState::Rejected; // TODO move to init

            let (tx2, mut rx) = tx_rx.pop().unwrap();
            let fut = tokio::spawn(async move {
                let mut level = 0;
                while !node.is_done() {
                    if !node.has_pending(){ // send one request at once
                        let edge = node.find_min_edge();
                        println!("{}) Found edge to={}", node, edge);
                        if node.states[edge] == EdgeState::Basic {
                            node.states[edge] = EdgeState::Requested;
                            let req = MessageRequest {
                                to: edge,
                                from: node.id,
                                level: level,
                                name: NAME, // FIXME
                                req: RequestType::Search,
                            };
                            tx2.send(req);
                            println!("{}) Requesting to={} l={}", node, req.to, level);
                        }
                    }
                    let received = rx.recv().await.unwrap(); // block until something happens
                    if received.to == node.id {
                        let edge_state = node.states[received.from];
                        match received.req {
                            RequestType::Search => {
                                match edge_state {
                                EdgeState::Requested => {
                                    // no need to answer as we already requested it
                                    // just merge, the other side will do the same
                                    if level == received.level {
                                        node.states[received.from] = EdgeState::Branch;
                                        println!("{}) MERGE {}", node, received.from);
                                        level = level + 1;
                                    } else if level > received.level {
                                        node.states[received.from] = EdgeState::Branch;
                                        println!("{}) ABSORB {}", node, received.from);
                                        let absorb = MessageRequest {
                                            to: received.from,
                                            from: node.id,
                                            level: level,
                                            req: RequestType::Absorb,
                                        };
                                        tx2.send(absorb);
                                    }else { // level < receied.level
                                        //TO EARLY TO SAY, we might get absorbed
                                        println!("{}) LEVEL DIFF {} from={}", node, received.level, received.from);
                                        //node.states[received.from] = EdgeState::Rejected;
                                    }
                                    //let req = MessageRequest {
                                    //    to: received.from,
                                    //    from: node.id,
                                    //    level: level,
                                    //    req: RequestType::Merge,
                                    //};
                                    //tx2.send(req);
                                },
                                EdgeState::Basic => {
                                    //println!("{}) received basic from={}", node.id, received.from);
                                    tx2.send(received); // FIXME
                                },
                                _ => panic!(),
                                };
                            }
                            RequestType::Absorb => { 
                                println!("{}) ABSORBED {}", node.id, received.from);
                                node.states[received.from] = EdgeState::Branch;
                                assert!(received.level > level);
                                level = received.level;
                            }
                            RequestType::Reject => {
                                node.states[received.from] = EdgeState::Rejected;
                            }
                        }
                    }
                }
                println!("{}) DONE {:?}", node.id, node.states);
            });
            handles.push(fut);
        }
        for handle in &mut handles {
            handle.await;
        }
        //thread::sleep_ms(500);
    }
}
