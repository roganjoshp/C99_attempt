use itertools::Itertools;
use rand::prelude::*;
use rayon::prelude::*;
use std::collections::HashSet;

const NODES: usize = 9;
const EDGES: usize = 4;

#[derive(Debug, Eq, PartialEq, Clone)]
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

#[derive(Debug, Default)]
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
    base_cost: f64,
    soln_cost: f64,
    best_ever_cost: f64,
    best_ever_solution: Graph,
    temperature: f64,
    alpha: f64,
    iterations: u64,
    latest_swap: (usize, usize, usize, usize),
}

impl<'a> Solver<'a> {
    fn new(graph: &'a mut Graph, temperature: f64, alpha: f64, iterations: u64) -> Self {
        Solver {
            graph: graph,
            base_cost: 10000.,
            soln_cost: 10000.,
            best_ever_cost: 10000.,
            best_ever_solution: Graph::default(),
            temperature: temperature,
            alpha: alpha,
            iterations: iterations,
            latest_swap: Default::default(),
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

    fn get_cost(&mut self) -> f64 {
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
                .count() as f64
                * 10.0
    }

    fn get_swaps(&mut self) -> Option<(usize, usize, usize, usize)> {
        let mut rng = rand::rng();
        // Probably easier/cheaper just to choose two random numbers
        // here in the range of node IDs and reject if they're equal
        // but... whatever for now
        let pair: Vec<&usize> = self.graph.node_ids.choose_multiple(&mut rng, 2).collect();

        // This would be a whole lot easier if you could get a diff
        // between sets without taking ownership...
        let mut swap_1 = usize::MIN;
        let mut swap_2 = usize::MIN;
        for &node_id in self.graph.nodes[*pair[0]].connections.iter() {
            if !self.graph.nodes[*pair[1]].connections.contains(&node_id) {
                swap_1 = node_id;
                break;
            }
        }

        for &node_id in self.graph.nodes[*pair[1]].connections.iter() {
            if !self.graph.nodes[*pair[0]].connections.contains(&node_id) {
                swap_2 = node_id;
                break;
            }
        }
        // Check whether this breaks an existing desired structure.
        // If it does, no touchy (for now, though I don't know
        // whether this procludes a better solution)
        // if self.neighbour_count_fits(swap_1, swap_2) {
        //     return None;
        // }

        Some((*pair[0], *pair[1], swap_1, swap_2))
    }

    fn run(&mut self) -> () {
        let mut rng = rand::rng();

        // Get the costs of the initial solution
        self.soln_cost = self.get_cost();
        self.best_ever_cost = self.get_cost();

        for x in 0..self.iterations {
            if let Some(swaps) = self.get_swaps() {
                self.latest_swap = swaps;
                self.graph.nodes[swaps.0].remove_connection(swaps.2);
                self.graph.nodes[swaps.1].remove_connection(swaps.3);
                self.graph.nodes[swaps.0].add_connection(swaps.3);
                self.graph.nodes[swaps.1].add_connection(swaps.2);
            } else {
                continue;
            }
            let new_cost = self.get_cost();
            if new_cost < self.soln_cost {
                // Accept unconditionally
                self.soln_cost = new_cost;
                if new_cost < self.best_ever_cost {
                    self.best_ever_cost = new_cost;
                    self.best_ever_solution.nodes = self.graph.nodes.clone();
                }
            } else {
                let calc =
                    (((self.soln_cost - new_cost) / self.soln_cost) * 100. / self.temperature);
                let check = f64::exp(calc);
                // println!("{:?}", check);
                let dice_roll: f64 = rng.random();
                if dice_roll < check && check != 1. {
                    self.soln_cost = new_cost;
                } else {
                    // We need to reverse the change
                    self.graph.nodes[self.latest_swap.0].remove_connection(self.latest_swap.3);
                    self.graph.nodes[self.latest_swap.1].remove_connection(self.latest_swap.2);
                    self.graph.nodes[self.latest_swap.0].add_connection(self.latest_swap.2);
                    self.graph.nodes[self.latest_swap.1].add_connection(self.latest_swap.3);
                }
            }
            self.temperature *= self.alpha;
        }
        println!("{:?}", self.best_ever_cost);
    }
}

fn main() {
    let mut graph = Graph::new();
    for x in 0..NODES {
        graph.add_node(Node::new(x));
    }

    graph.initialise_soln();
    println!("{:?}", graph.nodes);

    let mut solver = Solver::new(&mut graph, 0.4, 0.9995, 10000);
    solver.run();
}
