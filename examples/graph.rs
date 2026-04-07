//! Graph algorithms — Di Pierro Ch. 3.7
//! cargo run --example graph
use nlib::graph::{Graph, dijkstra, bfs, dfs, kruskal_mst};

fn main() {
    // Weighted graph (Di Pierro Fig. 3.12 style)
    //   0 --4-- 1 --2-- 2
    //   |       |       |
    //   1       3       5
    //   |       |       |
    //   3 --6-- 4 --1-- 5
    let mut g = Graph::new(6);
    g.add_undirected_edge(0, 1, 4.0);
    g.add_undirected_edge(1, 2, 2.0);
    g.add_undirected_edge(0, 3, 1.0);
    g.add_undirected_edge(1, 4, 3.0);
    g.add_undirected_edge(2, 5, 5.0);
    g.add_undirected_edge(3, 4, 6.0);
    g.add_undirected_edge(4, 5, 1.0);

    let dist = dijkstra(&g, 0);
    println!("Dijkstra from node 0:");
    for (i, d) in dist.iter().enumerate() {
        println!("  → node {i}: distance = {d}");
    }

    let bfs_order = bfs(&g, 0);
    println!("\nBFS from 0: {bfs_order:?}");

    let dfs_order = dfs(&g, 0);
    println!("DFS from 0: {dfs_order:?}");

    let mst = kruskal_mst(&g);
    let mst_weight: f64 = mst.iter().map(|e| e.2).sum();
    println!("\nKruskal MST (weight={mst_weight}):");
    for (u, v, w) in &mst {
        println!("  {u} -- {v}  (weight={w})");
    }
}
