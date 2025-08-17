import random
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
    """Assert that the solution is a strongly regular graph that meets criteria

    That is, all nodes must have the requisite number of connections in the
    graph and no node should be connected to itself

    Parameters
    ----------
    graph : np.ndarray
        A 2D array representing all edges between nodes

    Raises
    -------
    ValueError
        The graph does not comply with the restrictions
    """
    rows = (graph.sum(axis=0) != NUM_EDGES).sum()
    cols = (graph.sum(axis=1) != NUM_EDGES).sum()
    diag = graph.diagonal().sum()
    if rows != 0 or cols != 0 or diag != 0:
        raise ValueError("Graph is not compliant")


def get_score(graph: np.ndarray) -> tuple[int, int]:
    """Count the number of valid structures in the graph.

    The criteria for solving the problem stipulates "every pair of adjacent
    vertices should have 1 common neighbor, and every pair of non-adjacent
    vertices should have 2 common neighbors"

    This function countes the number of times that criteria is met

    Parameters
    ----------
    graph : np.ndarray
        A 2D array representing all edges between nodes

    Returns
    -------
    Tuple[int, int]
        A count of the number of [triangles, squares] in the solution structure
    """
    base = graph @ graph
    triangles = np.triu(base == 1, 0).sum()
    squares = np.triu(((base == 2) - graph).clip(0), 0).sum()
    return int(triangles), int(squares)


def solver(graph, iterations=10000):
    # Pre-compute nodes and acceptance criteria. Cheaper than calling random()
    # in a loop, even if we don't end up consuming them all
    node_selection = np.random.randint(0, NUM_NODES, size=iterations)
    pair_node_idx = np.random.randint(0, NUM_EDGES, size=iterations)
    acceptance = np.random.random(size=iterations)

    t, s = get_score(graph)
    current_score = 1000 - (t + s)
    best_ever_score = current_score
    solution = graph.copy()
    best_ever_solution = graph.copy()

    # Bind locally
    flatnonzero = np.flatnonzero

    for x in range(iterations):
        node = node_selection[x]
        pair_node = pair_node_idx[x]
        if node == pair_node:
            continue
        node_cons = set(flatnonzero(solution[node]))
        pair_node_cons = set(flatnonzero(solution[pair_node]))

        diff_f = pair_node_cons - node_cons
        diff_r = node_cons - pair_node_cons

        if node in diff_f:
            diff_f.remove(node)
        if pair_node in diff_r:
            diff_r.remove(pair_node)

        if not diff_f or not diff_r:
            continue

        f_node = random.choice(list(diff_f))
        r_node = random.choice(list(diff_r))

        # Remove the connections first
        solution[node][r_node] = 0
        solution[pair_node][f_node] = 0

        # Do the swap
        solution[node][f_node] = 1
        solution[pair_node][r_node] = 1
        if x % 100 == 0:
            assert_compliance(solution)

        t, s = get_score(solution)
        cost = 1000 - (t + s)

        if cost < current_score:
            if cost < best_ever_score:
                best_ever_score = cost
                best_ever_solution = solution.copy()
                print(x, best_ever_score)
        else:
            # Reverse
            solution[node][f_node] = 0
            solution[pair_node][r_node] = 0

            # Do the swap
            solution[node][r_node] = 1
            solution[pair_node][f_node] = 1


graph = initialise_graph(NUM_NODES, NUM_EDGES)
assert_compliance(graph)
print(graph)
score = get_score(graph)
solver(graph, iterations=10000)
