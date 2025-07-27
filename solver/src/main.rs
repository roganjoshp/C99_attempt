use itertools::Itertools;
use rand::prelude::*;
use rayon::prelude::*;
// use std::collections::HashMap;
use std::collections::HashSet;
// use std::rc::Rc;
// use std::sync::Arc;

const NODES: usize = 9;
const EDGES: usize = 4;

#[derive(Debug, Eq, PartialEq)]
struct Node {
    id: usize,
    connections: HashSet<usize>,
}

impl Node {
    fn new(id: usize) -> Self {
        Node {
            id: id,
            connections: HashSet::<usize>::new(),
        }
    }

    fn add_connection(&mut self, node_id: usize) -> () {
        self.connections.insert(node_id);
    }

    fn remove_connection(&mut self, node_id: usize) -> () {
        self.connections.remove(&node_id);
    }

    fn reset(&mut self) -> () {
        self.connections.clear();
    }
}

#[derive(Debug)]
struct Graph {
    nodes: Vec<Node>,
    node_ids: Vec<usize>,
    edges: Vec<(usize, usize)>,
    node_pairs: Vec<Vec<usize>>,
}

impl Graph {
    fn new() -> Self {
        Graph {
            nodes: Vec::new(),
            node_ids: Vec::with_capacity(NODES),
            edges: Vec::new(),
            node_pairs: (0..NODES).combinations(2).collect(),
        }
    }

    fn add_node(&mut self, node: Node) -> () {
        let node_id = node.id;
        self.nodes.insert(node.id, node);
        self.node_ids.push(node_id);
    }

    /// Attempt to generate a randomised initial solution
    /// that satisfies the number of nodes, each with the
    /// specified number of edges
    fn soln_initialiser(&mut self) -> () {
        // TODO: This is broken once you get to 99 nodes. It's just a POC.
        // Need to think about how to initialise this better
        let mut rng = rand::rng();

        for this_node_id in &self.node_ids {
            let mut pair_nodes: Vec<&usize> = self
                .node_ids
                .iter()
                .filter(|&n| n != this_node_id)
                .collect();

            pair_nodes.shuffle(&mut rng);

            for &pair_node_id in &pair_nodes {
                if self.nodes[*pair_node_id].connections.len() >= 4
                    || self.nodes[*this_node_id].connections.len() >= 4
                {
                    continue;
                }
                self.nodes[*pair_node_id].add_connection(*this_node_id);
                self.nodes[*this_node_id].add_connection(*pair_node_id);
                self.edges.push((*this_node_id, *pair_node_id));
            }
        }
    }

    /// Assert that the solution has the predicated number of
    /// edges for each node
    fn check_num_edges(&self) -> bool {
        for node in self.nodes.iter() {
            if node.connections.len() != EDGES {
                println!("Failed");
                return false;
            }
        }
        true
    }

    /// Generate the initial solution
    fn initialise_soln(&mut self) -> () {
        loop {
            self.soln_initialiser();
            if self.check_num_edges() {
                break;
            }
            for node_id in self.node_ids.iter() {
                self.nodes[*node_id].reset();
            }
        }
    }
}

struct Solver<'a> {
    graph: &'a mut Graph,
    base_cost: usize,
    soln_cost: usize,
    best_ever_cost: usize,
    temperature: f64,
    alpha: f64,
    iterations: u64,
}

impl<'a> Solver<'a> {
    fn new(graph: &'a mut Graph, temperature: f64, alpha: f64, iterations: u64) -> Self {
        Solver {
            graph: graph,
            base_cost: 1000,
            soln_cost: 1000,
            best_ever_cost: 1000,
            temperature: temperature,
            alpha: alpha,
            iterations: iterations,
        }
    }

    /// Check whether a node pair has 1 or 2 common connections
    /// Either the nodes are connected and share only a single
    /// connection, or they are not connected and share two node
    /// connections.
    fn neighbour_count_fits(&self, i: usize, j: usize) -> bool {
        // First count the mutual connections
        let count = self.graph.nodes[i]
            .connections
            .intersection(&self.graph.nodes[j].connections)
            .count();
        if count == 1 {
            // We need to be sure here that they are actually connected
            // themselves to create the triangle
            return self.graph.nodes[i].connections.contains(&j);
        }
        // The only way to make a square is if they share two connections
        // and aren't connected
        count == 2 && !self.graph.nodes[i].connections.contains(&j)
    }

    fn get_cost(&mut self) -> usize {
        // This is basically backwards from what I'm used to but
        // here I want to subtract from a static base cost for every
        // time I find a structure that I want in the graph.
        // Yam knows what weights to use to make SA be effective in
        // jumping out of local minima
        self.base_cost
            - self
                .graph
                .node_pairs
                .par_iter()
                .map(|pair| self.neighbour_count_fits(pair[0], pair[1]))
                .filter(|&s| s == true)
                .collect::<Vec<_>>()
                .len()
                * 10
    }
}

fn main() {
    let mut graph = Graph::new();
    for x in 0..NODES {
        graph.add_node(Node::new(x));
    }

    graph.initialise_soln();
    println!("{:?}", graph.nodes);

    let mut solver = Solver::new(&mut graph, 10.0, 0.9999, 1000);

    println!("{:?}", solver.get_cost());
}
