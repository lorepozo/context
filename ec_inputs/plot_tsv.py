import sys
import os
from math import exp
from functools import reduce

from matplotlib import pyplot as plt

if len(sys.argv) < 3:
    print("must supply two TSV files")
    exit(1)


PLOT = 'time' # 'time' or 'prob'
NAMES = ['primitive grammar', 'specialized grammar', 'contextual grammar']


def make_table(filename): # {taskname: (time, logprob)}
    tab = {}
    with open(filename) as f:
        for line in f:
            datum = line.strip().split("\t")
            name, time, logprob = datum[:3]
            tab[name] = (float(time), float(logprob))
        return tab

def ordered(tables, col, default=0, mapping=lambda x:x, ordering=lambda x:x, prob_ordering=False): # col into table value
    names = list(reduce(lambda x,y:x.union(y), (set(table.keys()) for table in tables)))
    for tab in tables[1:]+[tables[0]]:
        names.sort(key=lambda name: 0 if name not in tab else mapping(tab[name][col]), reverse=True)
    name_to_index = {name: i for i, name in enumerate(names)}
    N = len(name_to_index)

    def dimension(table):
        l = [default]*N
        for name in table:
            l[name_to_index[name]] = mapping(table[name][col])
        return l
    return names, list(map(dimension, tables))


tables = list(map(make_table, sys.argv[1:]))
T = len(tables)

bar_width = 0.7/T
plt.figure(figsize=(4,5))
if T % 2 == 1:
    offsets = list(range(-(T//2), 1+T//2))[::-1]
else:
    offsets = list(map(lambda x:x/2, range(1-T, T, 2)))[::-1]


if PLOT == 'time':
    names, values = ordered(tables, 0, mapping=lambda x:1/x)
    title = "task solve speed"
    xlabel = "solve speed (s⁻¹)"
elif PLOT == 'prob':
    names, values = ordered(tables, 1, mapping=exp, prob_ordering=True)
    title = "likelihood of solution"
    xlabel = "likelihood"
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
plt.savefig('fig.eps')
