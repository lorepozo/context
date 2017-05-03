import sys
import os
from math import exp
from functools import reduce
from itertools import combinations

from matplotlib import pyplot as plt

if len(sys.argv) < 6:
    print("must supply type and four TSV files")
    exit(1)


PLOT = sys.argv[1].lower() # 'speed_total', 'speed_iter' or 'likelihood'
NAMES = ['primitive grammar', 'specialized grammar per-phase', 'specialized grammar full-domain', 'contextual grammar']
NEG_INF = -10 # for logprob


TASK_NAMES = [ # used for ytick labels.
#sh: cat curriculum/all.json | jq -c '.tasks | .[]' | jq -r '.name'
"nospaces",
"first letters",
"concat first letters",
"filter cap",
"SICP",
"ruby -> r",
"find dot",
"find less-than",
"discard after dot",
"drop first two chars",
"discard before less-than",
"discard after greater-than",
"#include<os.h> -> os.h",
"#include<os.h> -> os",
"#include<os.h> -> OS",
"discard after @",
"dots to spaces",
"feach cap",
"ruby.clinton -> Ruby Clinton",
"ruby.clinton@mit.edu -> Ruby Clinton"
]


def make_table(filename): # {taskname: (time, logprob)}
    tab = {}
    with open(filename) as f:
        for line in f:
            datum = line.strip().split("\t")
            name, itertime, logprob, totaltime = datum[:4]
            tab[name] = (float(itertime), float(logprob), float(totaltime))
        return tab

def ordered(tables, col, # col into table value
        default=0, mapping=lambda x:x,
        ordering=lambda x:x, priority=lambda tbls:tbls[::-1]):
    names = list(reduce(lambda x,y:x.union(y), (set(table.keys()) for table in tables)))
    for tab in priority(tables)[::-1]:
        names.sort(key=lambda name: default if name not in tab else mapping(tab[name][col]), reverse=True)
    name_to_index = {name: i for i, name in enumerate(names)}
    N = len(name_to_index)

    def dimension(table):
        l = [default]*N
        for name in table:
            l[name_to_index[name]] = mapping(table[name][col])
        return l
    return names, list(map(dimension, tables))

def scatterify(values, err=0.2): # slightly adjusts values to prevent overlap in scatter plot
    def scatterify_iter(points):
        for pair in combinations(enumerate(points), 2):
            (_, a), (_, b) = pair
            if abs(a-b) < err:
                li, lp  = min(pair, key=lambda x:x[1])
                gi, gp = max(pair, key=lambda x:x[1])
                mp = (lp + gp)/2
                new_points = list(points)
                new_points[li] = mp - err*.55
                new_points[gi] = mp + err*.55
                return tuple(new_points), True
        return points, False
    for i, vs in enumerate(zip(*values)):
        had_err = True
        while had_err:
            vs, had_err = scatterify_iter(vs)
        for it in range(len(values)):
            values[it][i] = vs[it]
    return values


tables = list(map(make_table, sys.argv[2:]))
T = len(tables)

bar_width = 0.7/T
plt.figure(figsize=(4,5.8))
if T % 2 == 1:
    offsets = list(range(-(T//2), 1+T//2))[::-1]
else:
    offsets = list(map(lambda x:x/2, range(1-T, T, 2)))[::-1]


if PLOT == 'speed_total':
    names, values = ordered(tables, 2, mapping=lambda x:1/x)
    # failed tasks have zero speed
    inames, ivalues = ordered(tables, 0, mapping=lambda x:1/x)
    for ii, iname in enumerate(inames):
        i = names.index(iname)
        assert i != -1
        for it in range(T):
            if ivalues[it][i] == 0:
                values[it][i] = 0
    values = scatterify(values, err=0.05)
    title = "total task solve speed"
    xlabel = "solve speed (s⁻¹)"
    xticks = None
    scatter = True
elif PLOT == 'speed_iter':
    names, values = ordered(tables, 0, mapping=lambda x:1/x)
    values = scatterify(values, err=3)
    title = "final iteration task solve speed"
    xlabel = "solve speed (s⁻¹)"
    xticks = None
    scatter = True
elif PLOT == 'likelihood':
    names, values = ordered(tables, 1, default=NEG_INF)
    values = scatterify(values)
    title = "log likelihood of solution"
    xlabel = "log likelihood"
    xticks = (list(range(NEG_INF, 1, 2)), ['-∞']+list(map(str, range(NEG_INF+2, 1, 2))))
    scatter = True
else:
    raise ValueError("PLOT not valid")


N = len(names)
print(PLOT)
for i, name in enumerate(names):
    print(i, name)

for it in range(T):
    if scatter:
        for i in range(N):
            plt.axhline(y=i, color='lightgrey', zorder=-1)
        item = plt.scatter(values[it], list(range(N)))
    else:
        item = plt.barh(list(map(lambda i:i+bar_width*offsets[it], range(N))), values[it], bar_width)
    item.set_label(NAMES[it])
    if xticks:
        plt.xticks(*xticks)
plt.title(title)
plt.legend()
plt.xlabel(xlabel)
plt.yticks(range(N), list(map(lambda n: str(1+TASK_NAMES.index(n)), names)))
plt.ylabel("task")
plt.ylim(ymax=N+5)
plt.savefig(PLOT+'.eps')
