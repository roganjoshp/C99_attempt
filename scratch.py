# Need to test how to create an initial graph with the correct properties. I
# don't understand why my current approach is (semi-)broken and can deadlock
# itself.


NODE_COUNT = 9
EDGE_COUNT = 4


class Node:
    def __init__(self, id):
        self.id = id
        self.connections: set[Node] = set()

    def add_connection(self, other: "Node"):
        self.connections.add(other.id)

    def remove_connection(self, other: "Node"):
        self.connections = {
            node for node in self.connections if node != other.id
        }

    def get_connectedness(self, other: "Node"):
        return len(self.connections.intersection(other.connections))


class Graph:
    def __init__(self):
        self.graph = {}

    def add_node(self, node: Node):
        self.graph[node.id] = node

    def remove_node(self, node_id):
        edges = self.graph[node_id]
        for edge in edges:
            self.graph[edge].remove_connection(edge)
        del self.graph[node_id]

    def rank_nodes(self):
        # Clunky but just easier to debug visually and it's only used in
        # initialisation
        candidates = [
            (k, len(v.connections))
            for k, v in self.graph.items()
            if len(self.graph[k].connections) < EDGE_COUNT
        ]
        candidates.sort(key=lambda x: x[1])
        # print(candidates)
        return candidates

    def __repr__(self):
        conns = "\n".join(
            [f"{k} : {v.connections}" for k, v in self.graph.items()]
        )
        return conns


class Solver:
    def __init__(
        self, graph: Graph, temp: float, alpha: float, iterations: int
    ):
        self.graph = graph.graph
        self.temp = temp
        self.a = alpha
        self.iterations = iterations


"""
def generate_graph():
    pass
    

def generate_graph_greedy():
    all_nodes = [Node(i) for i in range(NODE_COUNT)]
    g = Graph()
    for node in all_nodes:
        g.add_node(node)
    g.rank_nodes()
    
    node_list = list(g.graph.keys())
    random.shuffle(node_list)
    
    for node in node_list:
        edges = 0
        while edges < EDGE_COUNT:
            node_index = 0

            pair_node = g.rank_nodes()[node_index][0]
            if node == pair_node:
                continue

            try:
                if (g.graph[node].get_connectedness(g.graph[pair_node]) 
                    < EDGE_COUNT):
                    g.graph[node].add_connection(g.graph[pair_node])
                    g.graph[pair_node].add_connection(g.graph[node])
                    edges += 1
                else:
                    node_index += 1
            except IndexError:
                g.graph[node].add_connection(g.graph[pair_node])
                g.graph[pair_node].add_connection(g.graph[node])
                edges += 1
"""

if __name__ == "__main__":
    pass  # generate_graph_greedy()
