use rand::prelude::*;
use std::collections::HashMap;
use std::collections::HashSet;

const NODES: u32 = 9;

#[derive(Debug)]
struct Graph {
    nodes: HashMap<u32, Node>,
    node_ids: Vec<u32>,
    edges: Vec<(u32, u32)>,
}

impl Graph {
    fn new() -> Self {
        Graph {
            nodes: HashMap::new(),
            node_ids: Vec::new(),
            edges: Vec::new(),
        }
    }

    fn add_node(&mut self, node: Node) -> () {
        let node_id = node.id;
        self.nodes.insert(node.id, node);
        self.node_ids.push(node_id);
    }

    fn initialse(&mut self) -> () {
        let mut rng = rand::rng();

        for this_node_id in &self.node_ids {
            let mut pair_nodes: Vec<&u32> = self
                .node_ids
                .iter()
                .filter(|&n| n != this_node_id)
                .collect();

            pair_nodes.shuffle(&mut rng);
            println!("{:?}", pair_nodes);

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
            }
        }
        println!("{:?}", self.nodes);
    }
}

#[derive(Debug, Eq, PartialEq)]
struct Node {
    id: u32,
    connections: HashSet<u32>,
}

impl Node {
    fn new(id: u32) -> Self {
        Node {
            id: id,
            connections: HashSet::<u32>::new(),
        }
    }

    fn add_connection(&mut self, node_id: u32) -> () {
        self.connections.insert(node_id);
    }
}

fn main() {
    let mut graph = Graph::new();
    for x in 0..NODES {
        graph.add_node(Node::new(x));
    }

    graph.initialse();

    // println!("{:?}", graph);
    // let mut rng = rand::rng();
    // let random_number: i32 = rng.random_range(2..4);
    // println!("Random number: {}", random_number);
}
