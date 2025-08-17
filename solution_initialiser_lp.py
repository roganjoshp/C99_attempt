from itertools import permutations

import numpy as np
import pulp
from pulp import LpProblem, LpVariable, lpSum, value

NUM_NODES = 9
NUM_EDGES = 4


def initialise_graph(nodes: int, edges: int) -> np.array:
    """
    Use linear programming to generate a template graph with required node:edge
    ratio
    """
    problem = LpProblem("initialisation")
    rows = cols = range(0, nodes)
    choices = LpVariable.dicts("choice", (rows, cols), cat="Binary")

    # The diagonal must be zero because nodes cannot connect to themselves
    for r in rows:
        problem += lpSum([choices[r][c] for c in cols if r != c]) == edges

    for c in cols:
        problem += lpSum([choices[r][c] for r in rows]) == edges

    # Try and force the graph to be undirected. Every reciprocal pairing of
    # nodes should cancel out. This should cover the case of 0-0 and 1-1 but
    # not allow for 1-0 or 0-1
    perms = permutations(range(NUM_NODES), 2)
    for perm in perms:
        # An added guard to not touch the diagonal. Probably not needed but meh
        if perm[0] == perm[1]:
            continue
        problem += (
            lpSum(choices[perm[0]][perm[1]] - choices[perm[1]][perm[0]]) == 0
        )

    if "COIN_CMD" in pulp.apis.listSolvers(onlyAvailable=True):
        # Needed for Mac users because the base binary is incompatible
        # see https://stackoverflow.com/a/79734034/4799172 for setup on Mac
        problem.solve(pulp.COIN_CMD("initialisation", msg=0))
    else:
        problem.solve(pulp.PULP_CBC_CMD("initialisation", msg=0))

    rebuilt = []
    for r in rows:
        row_vals = []
        for c in cols:
            val = value(choices[r][c])
            if val is not None:
                row_vals.append(int(val))
            else:
                row_vals.append(0)
        rebuilt.append(row_vals)

    return np.array(rebuilt).astype(int)


def assert_compliance(graph: np.ndarray):
    rows = (graph.sum(axis=0) != NUM_EDGES).sum()
    cols = (graph.sum(axis=1) != NUM_EDGES).sum()
    diag = graph.diagonal().sum()
    if rows != 0 or cols != 0 or diag != 0:
        raise ValueError("Graph is not compliant")


def get_score(graph: np.ndarray):
    base = graph @ graph
    triangles = np.triu(base == 1, 0).sum()
    squares = np.triu(((base == 2) - graph).clip(0), 0).sum()
    return int(triangles), int(squares)


graph = initialise_graph(NUM_NODES, NUM_EDGES)
assert_compliance(graph)
print(graph)
score = get_score(graph)
print(score)
