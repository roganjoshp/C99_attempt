from pulp import LpProblem, LpVariable, lpSum, value
from itertools import permutations
import pulp

import numpy as np


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
        problem += lpSum(
            choices[perm[0]][perm[1]] - choices[perm[1]][perm[0]]
        ) == 0

    if "COIN_CMD" in pulp.apis.listSolvers(onlyAvailable=True):
        # Needed for Mac users because the base binary is incompatible
        # see https://stackoverflow.com/a/79734034/4799172 for setup on Mac
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
            else:
                row_vals.append(0)
        rebuilt.append(row_vals)

    return np.array(rebuilt).astype(int)


graph = initialise_graph(NUM_NODES, NUM_EDGES)
print(graph)
