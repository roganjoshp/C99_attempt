from pulp import LpProblem, LpVariable, lpSum, value
import pulp

import numpy as np

np.set_printoptions(threshold=100000)


NUM_NODES = 9
NUM_EDGES = 4


class Node:
    def __init__(self, id: int):
        self.id = id
        self.connections: set[Node] = set()

    def add_connection(self, other: "Node"):
        self.connections.add(other.id)

    def remove_connection(self, other: "Node"):
        self.connections.remove(other.id)

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


class Solver:
    def __init__(self, graph: Graph):
        self.g = graph


def initialise_graph(nodes: int, edges: int) -> np.array:
    """
    Use linear programming to generate a template graph with required node:edge
    ratio
    """
    problem = LpProblem("initialisation")
    rows = cols = range(1, nodes)
    choices = LpVariable.dicts("choice", (rows, cols), cat="Binary")

    for r in rows:
        problem += lpSum([choices[r][c] for c in cols if c != r]) == edges

    for c in cols:
        problem += lpSum([choices[r][c] for r in rows if c != r]) == edges

    if "COIN_CMD" in pulp.apis.listSolvers(onlyAvailable=True):
        # Needed for Mac users because the base binary is incompatible
        # see https://stackoverflow.com/a/79734034/4799172
        problem.solve(pulp.COIN_CMD("initialisation"))
    else:
        problem.solve(pulp.PULP_CBC_CMD("initialisation"))

    rebuilt = []
    for r in rows:
        row_vals = []
        for c in cols:
            val = value(choices[r][c])
            if val is not None:
                row_vals.append(int(val))
                print(row_vals)
            else:
                row_vals.append(0)
        rebuilt.append(row_vals)

    return np.array(rebuilt).astype(int)


graph = initialise_graph(NUM_NODES, NUM_EDGES)
print(graph)
