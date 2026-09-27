//! Graph algorithms — Di Pierro, *Annotated Algorithms in Python*, §3.7.
//! Contract: `contracts/example-graph-v1.yaml` (entity: this example's `--json` receipt).
//!
//! cargo run --example graph            # human-readable
//! cargo run --example graph -- --json  # the receipt in evidence/examples/graph.json
use nlib::graph::{Graph, bfs, dfs, dijkstra, kruskal_mst};
use nlib::receipt::{Receipt, json_requested};

/// The example's undirected weighted edges:
///
/// ```text
///   0 --4-- 1 --2-- 2
///   |       |       |
///   1       3       5
///   |       |       |
///   3 --6-- 4 --1-- 5
/// ```
pub const EDGES: [(usize, usize, f64); 7] = [
    (0, 1, 4.0),
    (1, 2, 2.0),
    (0, 3, 1.0),
    (1, 4, 3.0),
    (2, 5, 5.0),
    (3, 4, 6.0),
    (4, 5, 1.0),
];

/// Shortest distances from node 0, by hand: 1 = 4, 2 = 4+2, 3 = 1, 4 = 4+3, 5 = 7+1.
pub const DIST_FROM_0: [f64; 6] = [0.0, 4.0, 6.0, 1.0, 7.0, 8.0];

/// Kruskal takes 0-3 (1), 4-5 (1), 1-2 (2), 1-4 (3), 0-1 (4): weight 11.
pub const MST_WEIGHT: f64 = 11.0;

/// True when no edge can shorten `d` (the Bellman optimality condition).
pub fn relaxed(d: &[f64], edges: &[(usize, usize, f64)]) -> bool {
    edges
        .iter()
        .all(|&(u, v, w)| d[v] <= d[u] + w && d[u] <= d[v] + w)
}

/// True when `order` visits every node of an `n`-node graph exactly once, starting at 0.
pub fn visits_all_once(order: &[usize], n: usize) -> bool {
    let mut seen = order.to_vec();
    seen.sort_unstable();
    order.first() == Some(&0) && seen == (0..n).collect::<Vec<_>>()
}

/// Every claim this example makes, as a receipt. Each check names the
/// `example-graph-v1` equation it exercises.
pub fn receipt(quiet: bool) -> Receipt {
    let say = |line: String| {
        if !quiet {
            println!("{line}");
        }
    };
    let mut r = Receipt::new("graph");
    let mut g = Graph::new(6);
    for &(u, v, w) in &EDGES {
        g.add_undirected_edge(u, v, w);
    }

    let dist = dijkstra(&g, 0);
    say("Dijkstra from node 0:".into());
    for (i, d) in dist.iter().enumerate() {
        say(format!("  → node {i}: distance = {d}"));
    }
    r.check("dijkstra_known", dist == DIST_FROM_0);
    r.check("dijkstra_relaxed", relaxed(&dist, &EDGES));

    let bfs_order = bfs(&g, 0);
    say(format!("\nBFS from 0: {bfs_order:?}"));
    r.check("bfs_visits_all", visits_all_once(&bfs_order, 6));
    let dfs_order = dfs(&g, 0);
    say(format!("DFS from 0: {dfs_order:?}"));
    r.check("dfs_visits_all", visits_all_once(&dfs_order, 6));

    let mst = kruskal_mst(&g);
    let weight: f64 = mst.iter().map(|e| e.2).sum();
    say(format!("\nKruskal MST (weight={weight}):"));
    for (u, v, w) in &mst {
        say(format!("  {u} -- {v}  (weight={w})"));
    }
    r.check("mst_spanning", mst.len() == 5);
    r.close("mst_weight_known", weight, MST_WEIGHT, 0.0);
    r
}

fn main() {
    let json = json_requested();
    std::process::exit(receipt(json).emit(json));
}
