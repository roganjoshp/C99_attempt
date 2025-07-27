use itertools::Itertools;
use rand::prelude::*;
use std::collections::HashMap;
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::Arc;

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
    nodes: HashMap<usize, Node>,
    node_ids: Vec<usize>,
    edges: Vec<(usize, usize)>,
    node_pairs: Vec<Vec<usize>>,
}

impl Graph {
    fn new() -> Self {
        Graph {
            nodes: HashMap::new(),
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
                if self.nodes.get(&pair_node_id).unwrap().connections.len() >= 4
                    || self.nodes.get(&this_node_id).unwrap().connections.len() >= 4
                {
                    continue;
                }
                self.nodes
                    .get_mut(&pair_node_id)
                    .unwrap()
                    .add_connection(*this_node_id);
                self.nodes
                    .get_mut(this_node_id)
                    .unwrap()
                    .add_connection(*pair_node_id);
                self.edges.push((*this_node_id, *pair_node_id));
            }
        }
    }

    /// Assert that the solution has the predicated number of
    /// edges for each node
    fn check_num_edges(&self) -> bool {
        for (_, node) in self.nodes.iter() {
            if node.connections.len() != EDGES {
                println!("Failed initialisation");
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
                self.nodes.get_mut(node_id).unwrap().reset();
            }
        }
    }
}

struct Solver<'a> {
    graph: &'a mut Graph,
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
            soln_cost: 1000,
            best_ever_cost: 1000,
            temperature: temperature,
            alpha: alpha,
            iterations: iterations,
        }
    }

    fn get_cost(&mut self) -> () {}
}

fn main() {
    let mut graph = Graph::new();
    for x in 0..NODES {
        graph.add_node(Node::new(x));
    }

    graph.initialise_soln();

    let solver = Solver::new(&mut graph, 10.0, 0.9999, 1000);

    println!("{:?}", graph.nodes);
}
