pub mod distributed_ghs_mst;

use distributed_ghs_mst::*;

#[tokio::main]
async fn main() {
    let vectored = TEST_DATA2.iter().map(|arr| arr.to_vec()).collect();
    let mut graph = Graph::new(vectored);
    let fut = graph.start();
    fut.await;
    
    //let mut rt = tokio::runtime::Runtime::new().unwrap();
    //rt.block_on(
}
