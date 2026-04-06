//! Graph algorithms — contract: `graph-algorithms-v1.yaml`
//!
//! Di Pierro Ch. 11: Dijkstra, BFS, DFS, Kruskal MST.
//! Uses `aprender::graph::Graph` (CSR) as the reference graph type
//! and maintains a local adjacency-list `Graph` for nlib's mutable API.

use std::collections::VecDeque;
use aprender::graph::Graph as AprGraph;

/// Weighted directed graph (adjacency list).
#[derive(Debug, Clone)]
pub struct Graph {
    pub n: usize,
    adj: Vec<Vec<(usize, f64)>>,
}

impl Graph {
    pub fn new(n: usize) -> Self { Self { n, adj: vec![vec![]; n] } }
    pub fn add_edge(&mut self, u: usize, v: usize, w: f64) {
        assert!(u < self.n && v < self.n, "vertex out of range");
        self.adj[u].push((v, w));
    }
    pub fn add_undirected_edge(&mut self, u: usize, v: usize, w: f64) {
        self.add_edge(u, v, w);
        self.add_edge(v, u, w);
    }
    pub fn neighbors(&self, u: usize) -> &[(usize, f64)] { &self.adj[u] }

    /// Convert to aprender's CSR Graph for analysis.
    pub fn to_aprender(&self) -> AprGraph {
        let mut edges = Vec::new();
        for u in 0..self.n {
            for &(v, w) in &self.adj[u] {
                edges.push((u, v, w));
            }
        }
        if edges.is_empty() {
            // aprender needs at least self-edges to know node count
            AprGraph::from_edges(
                &(0..self.n).map(|i| (i, i)).collect::<Vec<_>>(),
                true,
            )
        } else {
            AprGraph::from_weighted_edges(&edges, true)
        }
    }
}

/// Dijkstra shortest-path. Unreachable vertices get f64::INFINITY.
pub fn dijkstra(graph: &Graph, start: usize) -> Vec<f64> {
    assert!(start < graph.n, "start out of range");
    for u in 0..graph.n {
        for &(_, w) in graph.neighbors(u) {
            assert!(w >= 0.0, "dijkstra: negative weight not allowed");
        }
    }
    let n = graph.n;
    let mut dist = vec![f64::INFINITY; n];
    let mut visited = vec![false; n];
    dist[start] = 0.0;
    for _ in 0..n {
        let mut u = None;
        let mut min_d = f64::INFINITY;
        for v in 0..n {
            if !visited[v] && dist[v] < min_d { min_d = dist[v]; u = Some(v); }
        }
        let u = match u { Some(v) => v, None => break };
        visited[u] = true;
        for &(v, w) in graph.neighbors(u) {
            let alt = dist[u] + w;
            if alt < dist[v] { dist[v] = alt; }
        }
    }
    dist
}

/// BFS traversal order from `start`.
pub fn bfs(graph: &Graph, start: usize) -> Vec<usize> {
    assert!(start < graph.n);
    let mut visited = vec![false; graph.n];
    let mut queue = VecDeque::new();
    let mut order = Vec::new();
    visited[start] = true;
    queue.push_back(start);
    while let Some(u) = queue.pop_front() {
        order.push(u);
        let mut nbrs: Vec<usize> = graph.neighbors(u).iter().map(|&(v, _)| v).collect();
        nbrs.sort();
        for v in nbrs {
            if !visited[v] { visited[v] = true; queue.push_back(v); }
        }
    }
    order
}

/// DFS traversal order from `start` (iterative).
pub fn dfs(graph: &Graph, start: usize) -> Vec<usize> {
    assert!(start < graph.n);
    let mut visited = vec![false; graph.n];
    let mut stack = vec![start];
    let mut order = Vec::new();
    while let Some(u) = stack.pop() {
        if visited[u] { continue; }
        visited[u] = true;
        order.push(u);
        let mut nbrs: Vec<usize> = graph.neighbors(u).iter().map(|&(v, _)| v).collect();
        nbrs.sort();
        for v in nbrs.into_iter().rev() {
            if !visited[v] { stack.push(v); }
        }
    }
    order
}

/// Kruskal MST. Returns edges (u, v, weight).
pub fn kruskal_mst(graph: &Graph) -> Vec<(usize, usize, f64)> {
    let n = graph.n;
    let mut edges: Vec<(usize, usize, f64)> = Vec::new();
    for u in 0..n {
        for &(v, w) in graph.neighbors(u) {
            if u < v { edges.push((u, v, w)); }
        }
    }
    edges.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap());
    let mut parent: Vec<usize> = (0..n).collect();
    let mut rank = vec![0usize; n];
    let mut mst = Vec::new();
    for (u, v, w) in edges {
        let (ru, rv) = (find(&mut parent, u), find(&mut parent, v));
        if ru != rv {
            union(&mut parent, &mut rank, ru, rv);
            mst.push((u, v, w));
        }
        if mst.len() == n - 1 { break; }
    }
    mst
}

fn find(p: &mut [usize], mut x: usize) -> usize {
    while p[x] != x { p[x] = p[p[x]]; x = p[x]; }
    x
}
fn union(p: &mut [usize], r: &mut [usize], x: usize, y: usize) {
    if r[x] < r[y] { p[x] = y; } else { p[y] = x; if r[x] == r[y] { r[x] += 1; } }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn triangle() -> Graph {
        let mut g = Graph::new(3);
        g.add_undirected_edge(0, 1, 1.0);
        g.add_undirected_edge(1, 2, 2.0);
        g.add_undirected_edge(0, 2, 3.0);
        g
    }
    fn five_node() -> Graph {
        let mut g = Graph::new(5);
        g.add_undirected_edge(0, 1, 1.0);
        g.add_undirected_edge(1, 2, 2.0);
        g.add_undirected_edge(0, 3, 4.0);
        g.add_undirected_edge(1, 4, 3.0);
        g.add_undirected_edge(2, 4, 1.0);
        g.add_undirected_edge(3, 4, 5.0);
        g
    }
    #[test] fn dijkstra_single() {
        assert!((dijkstra(&Graph::new(1), 0)[0]).abs() < 1e-10);
    }
    #[test] fn dijkstra_tri() {
        let d = dijkstra(&triangle(), 0);
        assert!((d[0]).abs() < 1e-10 && (d[1] - 1.0).abs() < 1e-10 && (d[2] - 3.0).abs() < 1e-10);
    }
    #[test] fn dijkstra_five() {
        let d = dijkstra(&five_node(), 0);
        assert!((d[1] - 1.0).abs() < 1e-10 && (d[2] - 3.0).abs() < 1e-10);
        assert!((d[3] - 4.0).abs() < 1e-10 && (d[4] - 4.0).abs() < 1e-10);
    }
    #[test] fn dijkstra_unreachable() {
        let mut g = Graph::new(3); g.add_edge(0, 1, 1.0);
        assert!(dijkstra(&g, 0)[2].is_infinite());
    }
    #[test] #[should_panic] fn dijkstra_neg() {
        let mut g = Graph::new(2); g.add_edge(0, 1, -1.0); dijkstra(&g, 0);
    }
    #[test] fn bfs_tri() {
        let o = bfs(&triangle(), 0);
        assert_eq!(o[0], 0); assert_eq!(o.len(), 3);
    }
    #[test] fn bfs_disconnected() {
        let mut g = Graph::new(4);
        g.add_undirected_edge(0, 1, 1.0); g.add_undirected_edge(2, 3, 1.0);
        let o = bfs(&g, 0);
        assert!(o.contains(&0) && o.contains(&1) && !o.contains(&2));
    }
    #[test] fn bfs_level_order() {
        let o = bfs(&five_node(), 0);
        assert!(o.iter().position(|&v| v == 0).unwrap() < o.iter().position(|&v| v == 1).unwrap());
    }
    #[test] fn dfs_tri() {
        let o = dfs(&triangle(), 0);
        assert_eq!(o[0], 0); assert_eq!(o.len(), 3);
    }
    #[test] fn dfs_all_reachable() { assert_eq!(dfs(&five_node(), 0).len(), 5); }
    #[test] fn dfs_disconnected() { assert_eq!(dfs(&Graph::new(3), 0), vec![0]); }
    #[test] fn kruskal_tri() {
        let mst = kruskal_mst(&triangle());
        assert_eq!(mst.len(), 2);
        let total: f64 = mst.iter().map(|e| e.2).sum();
        assert!((total - 3.0).abs() < 1e-10);
    }
    #[test] fn kruskal_five() {
        let mst = kruskal_mst(&five_node());
        assert_eq!(mst.len(), 4);
        let total: f64 = mst.iter().map(|e| e.2).sum();
        assert!((total - 8.0).abs() < 1e-10);
    }
    #[test] fn kruskal_single() { assert!(kruskal_mst(&Graph::new(1)).is_empty()); }
    #[test] fn kruskal_edge_count() {
        assert_eq!(kruskal_mst(&five_node()).len(), 4);
    }
    #[test] fn to_aprender_conversion() {
        let g = triangle();
        let apr = g.to_aprender();
        assert!(apr.num_nodes() >= 3);
    }
}
