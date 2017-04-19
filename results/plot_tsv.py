import sys
import os
from math import exp
from functools import reduce

from matplotlib import pyplot as plt

if len(sys.argv) < 6:
    print("must supply type and four TSV files")
    exit(1)


PLOT = sys.argv[1].lower() # 'speed' or 'likelihood'
NAMES = ['primitive grammar', 'per-task specialized grammar', 'full-domain specialized grammar', 'contextual grammar']
NEG_INF = -10 # for logprob


def make_table(filename): # {taskname: (time, logprob)}
    tab = {}
    with open(filename) as f:
        for line in f:
            datum = line.strip().split("\t")
            name, time, logprob = datum[:3]
            tab[name] = (float(time), float(logprob))
        return tab

def ordered(tables, col, default=0, mapping=lambda x:x, ordering=lambda x:x): # col into table value
    names = list(reduce(lambda x,y:x.union(y), (set(table.keys()) for table in tables)))
    for tab in tables[1:]+[tables[0]]:
        names.sort(key=lambda name: default if name not in tab else mapping(tab[name][col]), reverse=True)
    name_to_index = {name: i for i, name in enumerate(names)}
    N = len(name_to_index)

    def dimension(table):
        l = [default]*N
        for name in table:
            l[name_to_index[name]] = mapping(table[name][col])
        return l
    return names, list(map(dimension, tables))


tables = list(map(make_table, sys.argv[2:]))
T = len(tables)

bar_width = 0.7/T
plt.figure(figsize=(4,5))
if T % 2 == 1:
    offsets = list(range(-(T//2), 1+T//2))[::-1]
else:
    offsets = list(map(lambda x:x/2, range(1-T, T, 2)))[::-1]


if PLOT == 'speed':
    names, values = ordered(tables, 0, mapping=lambda x:1/x)
    title = "task solve speed"
    xlabel = "solve speed (s⁻¹)"
elif PLOT == 'likelihood':
    names, values = ordered(tables, 1)
    title = "likelihood of solution"
    xlabel = "likelihood"
    #xticks = (range(NEG_INF, 1), ['-∞']+range(NEG_INF+1, 1, 2))
else:
    raise ValueError("PLOT not valid")


N = len(names)
print(PLOT)
for i, name in enumerate(names):
    print(i, name)

for it in range(T):
    plt.barh(list(map(lambda i:i+bar_width*offsets[it], range(N))), values[it], bar_width)
plt.title(title)
plt.xlabel(xlabel)
plt.legend(NAMES)
plt.yticks(range(N),['']*N)
plt.ylabel("task")
plt.savefig(PLOT+'.eps')
