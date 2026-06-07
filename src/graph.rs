//! Call-graph construction and rendering.
//!
//! Given the flat set of caller->callee edges from the index, we do a
//! breadth-first traversal from a root symbol to a bounded depth and render
//! the result as an indented tree, Graphviz DOT, or Mermaid.

use std::collections::{BTreeSet, HashMap, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Follow edges forward: what does the root call (and what do those call)?
    Callees,
    /// Follow edges backward: who calls the root (and who calls them)?
    Callers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Tree,
    Dot,
    Mermaid,
}

/// Adjacency built once from the edge list, in both directions.
pub struct Graph {
    forward: HashMap<String, BTreeSet<String>>, // caller -> callees
    backward: HashMap<String, BTreeSet<String>>, // callee -> callers
}

impl Graph {
    pub fn from_edges(edges: &[(String, String)]) -> Graph {
        let mut forward: HashMap<String, BTreeSet<String>> = HashMap::new();
        let mut backward: HashMap<String, BTreeSet<String>> = HashMap::new();
        for (caller, callee) in edges {
            forward
                .entry(caller.clone())
                .or_default()
                .insert(callee.clone());
            backward
                .entry(callee.clone())
                .or_default()
                .insert(caller.clone());
        }
        Graph { forward, backward }
    }

    fn neighbors(&self, node: &str, dir: Direction) -> Option<&BTreeSet<String>> {
        match dir {
            Direction::Callees => self.forward.get(node),
            Direction::Callers => self.backward.get(node),
        }
    }

    /// Breadth-first set of reachable edges from `root` up to `max_depth`,
    /// returned as a deduplicated, ordered edge list for rendering. Cycles are
    /// handled by not re-expanding an already-visited node.
    pub fn reachable_edges(
        &self,
        root: &str,
        dir: Direction,
        max_depth: usize,
    ) -> Vec<(String, String)> {
        let mut visited: BTreeSet<String> = BTreeSet::new();
        let mut emitted: BTreeSet<(String, String)> = BTreeSet::new();
        let mut out: Vec<(String, String)> = Vec::new();
        let mut queue: VecDeque<(String, usize)> = VecDeque::new();
        queue.push_back((root.to_string(), 0));
        visited.insert(root.to_string());

        while let Some((node, depth)) = queue.pop_front() {
            if depth >= max_depth {
                continue;
            }
            if let Some(neighbors) = self.neighbors(&node, dir) {
                for n in neighbors {
                    // Normalize edge orientation to (caller, callee) regardless
                    // of traversal direction, so DOT/Mermaid arrows read right.
                    let edge = match dir {
                        Direction::Callees => (node.clone(), n.clone()),
                        Direction::Callers => (n.clone(), node.clone()),
                    };
                    if emitted.insert(edge.clone()) {
                        out.push(edge);
                    }
                    if visited.insert(n.clone()) {
                        queue.push_back((n.clone(), depth + 1));
                    }
                }
            }
        }
        out
    }
}

/// Render the graph in the requested format. `root`, `dir`, and `edges` are
/// the already-computed reachable edges.
pub fn render(root: &str, edges: &[(String, String)], dir: Direction, fmt: Format) -> String {
    match fmt {
        Format::Dot => render_dot(root, edges),
        Format::Mermaid => render_mermaid(edges),
        Format::Tree => render_tree(root, edges, dir),
    }
}

fn sanitize(id: &str) -> String {
    // DOT/Mermaid node ids must be alphanumeric-ish; quote the label instead.
    id.chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect()
}

fn render_dot(root: &str, edges: &[(String, String)]) -> String {
    let mut s = String::from("digraph callgraph {\n");
    s.push_str("  rankdir=LR;\n");
    s.push_str("  node [shape=box, fontname=\"monospace\"];\n");
    s.push_str(&format!(
        "  {} [label=\"{}\", style=filled, fillcolor=\"#cde\"];\n",
        sanitize(root),
        root
    ));
    for (a, b) in edges {
        s.push_str(&format!(
            "  {} [label=\"{}\"];\n  {} [label=\"{}\"];\n  {} -> {};\n",
            sanitize(a),
            a,
            sanitize(b),
            b,
            sanitize(a),
            sanitize(b)
        ));
    }
    s.push_str("}\n");
    s
}

fn render_mermaid(edges: &[(String, String)]) -> String {
    let mut s = String::from("graph LR\n");
    for (a, b) in edges {
        s.push_str(&format!(
            "  {}[\"{}\"] --> {}[\"{}\"]\n",
            sanitize(a),
            a,
            sanitize(b),
            b
        ));
    }
    s
}

fn render_tree(root: &str, edges: &[(String, String)], dir: Direction) -> String {
    // Rebuild adjacency limited to the reachable edge set for a clean tree.
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();
    for (a, b) in edges {
        match dir {
            Direction::Callees => adj.entry(a.clone()).or_default().push(b.clone()),
            Direction::Callers => adj.entry(b.clone()).or_default().push(a.clone()),
        }
    }
    let mut out = String::new();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    write_tree(root, &adj, &mut out, "", true, &mut seen);
    out
}

fn write_tree(
    node: &str,
    adj: &HashMap<String, Vec<String>>,
    out: &mut String,
    prefix: &str,
    is_root: bool,
    seen: &mut BTreeSet<String>,
) {
    if is_root {
        out.push_str(node);
        out.push('\n');
    }
    // Cycle guard: track the *current path* (ancestors), not all nodes ever
    // seen. A node reached again via a different branch (a diamond) is fine;
    // only a node that is its own ancestor is a real cycle.
    if !seen.insert(node.to_string()) {
        out.push_str(&format!("{prefix}(recursion: {node})\n"));
        return;
    }
    if let Some(children) = adj.get(node) {
        let mut kids = children.clone();
        kids.sort();
        kids.dedup();
        let n = kids.len();
        for (i, child) in kids.iter().enumerate() {
            let last = i + 1 == n;
            let branch = if last { "└── " } else { "├── " };
            out.push_str(&format!("{prefix}{branch}{child}\n"));
            let next_prefix = format!("{prefix}{}", if last { "    " } else { "│   " });
            write_tree(child, adj, out, &next_prefix, false, seen);
        }
    }
    // Leaving this node: pop it from the current path so sibling branches can
    // legitimately reach it without being flagged as recursion.
    seen.remove(node);
}
