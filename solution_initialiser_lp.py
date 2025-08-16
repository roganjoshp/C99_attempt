from pulp import LpProblem, LpVariable, lpSum, value
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
    rows = cols = range(1, nodes)
    choices = LpVariable.dicts("choice", (rows, cols), cat="Binary")

    for r in rows:
        problem += lpSum([choices[r][c] for c in cols]) == edges

    for c in cols:
        problem += lpSum([choices[r][c] for r in rows]) == edges

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
            row_vals.append(int(value(choices[r][c])))
        rebuilt.append(row_vals)
    return np.array(rebuilt)


graph = initialise_graph(NUM_NODES, NUM_EDGES)
print(graph)
