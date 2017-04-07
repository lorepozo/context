extern crate rand;

use std::io::Write;
use std::cmp::{min, max};
use std::collections::{HashSet, HashMap};
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use rand::distributions::{IndependentSample, Range};

const CTX_MIN_SIZE: usize = 3;
const NET_MAX_SIZE: usize = 128;

/// Item maintains the data and metadata for a single knowledge artifact.
#[derive(Debug)]
struct Item {
    /// mechanism name
    mech: &'static str,
    /// arbitrary data
    data: Rc<String>,
    /// counts maps an epoch to a number of accesses to this artifact made
    /// during that epoch.
    counts: HashMap<usize, u64>,
    /// adj is the set of adjacent item ids.
    adj: HashSet<usize>,
    /// id is this item's unique identifier.
    id: usize,
}

impl Item {
    fn new(mech: &'static str, adj: HashSet<usize>, data: String, id: usize) -> Item {
        Item {
            mech: mech,
            data: Rc::new(data),
            counts: HashMap::new(),
            adj: adj,
            id: id,
        }
    }
    /// increases this item's access count for a given epoch.
    fn add_count(&mut self, epoch: usize, count: u64) {
        let prev_count: u64 = {
            *self.counts.get(&epoch).unwrap_or(&0)
        };
        self.counts.insert(epoch, count + prev_count);
    }
    /// gets the count of accesses to this item since the given epoch.
    fn recent_count(&self, epoch: usize) -> u64 {
        self.counts.iter().filter(|&(e, _)| *e >= epoch).fold(0, |s, (_, c)| s + c)
    }
}

/// Context is the interface for mechanisms to utilize the knowledge
/// network.
pub struct Context {
    /// net is the network that the context corresponds to.
    net: Network,
    /// name of the mechanism that's using this particular Context object.
    mech: &'static str,
    /// the set of item ids in the immediate context.
    items: HashSet<usize>,
    /// the set of item ids within a small boundary over the immediate
    /// context.
    frontier: HashSet<usize>,
    /// the epoch that this Context object was created in.
    initial_epoch: usize,
    /// the epoch that this Context currently corresponds to.
    current_epoch: usize,
}

impl Context {
    pub fn add_item_count(&self, id: usize, count: u64) {
        self.net.item_count(self.current_epoch, id, count)
    }
    pub fn get(&self) -> Vec<(usize, &'static str, Rc<String>)> {
        self.net.ids_to_contents(self.items.clone())
    }
    pub fn explore(&self) -> Vec<(usize, &'static str, Rc<String>)> {
        self.net.ids_to_contents(self.items.union(&self.frontier).cloned())
    }
    /// update will give a new Context object that accounts for any changes
    /// that may have happened (such as from .orient() or .grow()) since
    /// this Context object was created.
    pub fn update(&self) -> Context {
        Context { initial_epoch: self.initial_epoch, ..self.net.context(self.mech) }
    }
    pub fn orient(&self, id: usize) {
        self.net.orient(self.initial_epoch, id)
    }
    pub fn grow_for_mech(&self, mech: &'static str, data: String) -> usize {
        self.net.grow(mech, data, self.initial_epoch)
    }
    pub fn grow(&self, data: String) -> usize {
        self.grow_for_mech(self.mech, data)
    }
}

#[derive(Debug)]
struct Net {
    /// The minimum context size, used only if larger than embryo size (otherwise the embryo size
    /// is used). The context size may grow from this amount as the network grows.
    context_min_size: usize,
    /// For obvious reasons, an upper-bound may be set on the network size.
    max_size: usize,
    /// Graph maps id -> item.
    graph: Vec<Item>,
    /// Epochs records the id of each orient call (context-switch), its
    /// associated context, and the ids of accessed items in that epoch.
    epochs: Vec<(usize, HashSet<usize>, HashSet<usize>)>,
}

#[derive(Clone)]
struct Network {
    net: Rc<RefCell<Net>>,
}

impl Network {
    /// embryo is a collection of starting items to form an initial clique
    /// graph, of the form (mechanism name, data). Must be non-empty.
    pub fn new<U>(embryo: U) -> Network
        where U: IntoIterator<Item = (&'static str, String)>
    {
        let network = Network {
            net: Rc::new(RefCell::new(Net {
                context_min_size: CTX_MIN_SIZE,
                max_size: NET_MAX_SIZE,
                graph: Vec::new(),
                epochs: Vec::new(),
            })),
        };
        {
            // scope in for this mutable borrow
            let mut net = network.net.borrow_mut();
            // clique of embryo as base
            let mut id = 0;
            let embryo: Vec<(&'static str, String)> = embryo.into_iter().collect();
            let edges: HashSet<usize> = (0..embryo.len()).collect();
            net.graph = embryo.into_iter()
                .map(|(mech, data)| {
                    let mut edges = edges.clone();
                    edges.remove(&id);
                    let item = Item::new(mech, edges, data, id);
                    id = id + 1;
                    item
                })
                .collect();
            // initial epoch has no accesses and context of entire embyro
            net.epochs.push((0, edges, HashSet::new())); // edges ~ embryo ids
            // initial size
            let size = id;
            assert_ne!(0, size);
            if net.context_min_size < size {
                net.context_min_size = size;
            }
        }
        network
    }
    /// item_count increases the count of a given item corresponding to the
    /// given epoch.
    fn item_count(&self, epoch: usize, id: usize, count: u64) {
        let mut net = self.net.borrow_mut();
        {
            let ref mut item = net.graph[id];
            item.add_count(epoch, count);
        }
        net.epochs[epoch].2.insert(id);
    }
    /// ids_to_contexts takes an iterable of item ids and returns a vector
    /// of (id, mechanism name, data) corresponding to each given id.
    fn ids_to_contents<U>(&self, items: U) -> Vec<(usize, &'static str, Rc<String>)>
        where U: IntoIterator<Item = usize>
    {
        let net = self.net.borrow();
        items.into_iter()
            .map(move |id| {
                let ref item = net.graph[id];
                (id, item.mech, item.data.clone())
            })
            .collect()
    }
    /// orient creates a new epoch, centering the context around the given
    /// item and using items' access counts since the given epoch to
    /// determine where to grow the context.
    fn orient(&self, epoch: usize, id: usize) {
        let mut net = self.net.borrow_mut();
        let mut ctx: HashSet<usize>;
        let n = net.graph.len();
        if n < net.context_min_size {
            // use the entire network
            ctx = (0..n).collect();
        } else {
            // the context should be sized according to the expected max
            // degree of a scale-free network with size n:
            //  k_max ~ n^{1/(gamma-1)}
            // for degree exponent 2 < gamma < 3.
            let mut rng = rand::thread_rng();
            let exp = Range::new(0.5f64, 1f64).ind_sample(&mut rng);
            let prop = 0.5;
            let kmax = prop * (n as f64).powf(exp);
            let size = max(net.context_min_size, min(n, kmax as usize));

            // start with the given node. add its neighbors in order by
            // popularity, select most popular neighbor and repeat. stop
            // when we get to `size`.
            ctx = HashSet::new();
            let mut selected = id;
            while ctx.len() < size {
                let mut ext: Vec<usize> = net.graph[selected]
                    .adj
                    .difference(&ctx)
                    .cloned()
                    .collect();
                if ext.is_empty() {
                    break;
                }
                if ext.len() + ctx.len() > size {
                    // truncate less-used items
                    let take = size - ctx.len();
                    ext.sort_by_key(|&id| {
                        let ref item = net.graph[id];
                        -(item.recent_count(epoch) as i64) // reversed
                    });
                    ctx.extend(ext.iter().take(take));
                    break;
                }
                selected = ext.iter()
                    .max_by_key(|&id| {
                        let ref item = net.graph[*id];
                        item.recent_count(epoch)
                    })
                    .unwrap()
                    .clone();
                ctx.extend(ext);
            }
        }
        net.epochs.push((id, ctx, HashSet::new()))
    }
    /// grow adds a new knowledge artifact (Item) to the network, and
    /// creates a new epoch with an implicit call to .orient() on the new
    /// item.
    fn grow(&self, mech: &'static str, data: String, epoch: usize) -> usize {
        let id: usize;
        {
            let mut net = self.net.borrow_mut();
            id = net.graph.len();
            assert!(id <= net.max_size);

            // compute counts for antecedent artifacts
            let ids: HashSet<usize> = net.epochs
                .iter()
                .skip(epoch) // look for accesses as early as this epoch
                .flat_map(|&(_, ref cx, ref ru)| cx.union(ru).cloned())
                .collect(); // removes duplicates
            let mut sum = 0;
            let counts: HashSet<(usize, u64)> = ids.iter()
                .map(|&id| {
                    let ref item = net.graph[id];
                    let count = 1 + item.recent_count(epoch);
                    sum += count;
                    (id, count)
                })
                .collect();

            // convert counts to probabilities
            let antecedents: HashMap<usize, f64> = counts.iter()
                .map(|&(id, cnt)| {
                    let p = (cnt as f64) / (sum as f64);
                    (id, p)
                })
                .collect();
            let mut edges = HashSet::new();

            // popularity-based subset selection
            let uniform = Range::new(0f64, 1.);
            let mut rng = rand::thread_rng();
            for (id, p) in &antecedents {
                let sample = uniform.ind_sample(&mut rng);
                let connect = sample <= *p;
                if connect {
                    edges.insert(*id);
                }
            }
            // CRP if we got an empty subset
            if edges.is_empty() {
                let mut r = uniform.ind_sample(&mut rng);
                for (id, p) in &antecedents {
                    r -= *p;
                    if r < 0f64 {
                        edges.insert(*id);
                        break;
                    }
                }
            }

            // update other end of new edges (undirected network)
            for oid in &edges {
                let ref mut item = net.graph[*oid];
                item.adj.insert(id);
            }

            // actually add the item
            let item = Item::new(mech, edges, data, id);
            net.graph.push(item);
        }
        self.orient(epoch, id);
        id
    }
    /// frontier_of takes a set of item ids and returns the set of item ids
    /// corresponding to all adjacent items.
    fn frontier_of(&self, items: &HashSet<usize>) -> HashSet<usize> {
        let net = self.net.borrow();
        let frontier: HashSet<usize> = items.iter()
            .flat_map(|&id| {
                let ref item = net.graph[id];
                item.adj.clone()
            })
            .collect();
        frontier.difference(items).cloned().collect()
    }
    /// context creates a new Context object corresponding to the network's
    /// latest epoch.
    fn context(&self, mech: &'static str) -> Context {
        let net = self.net.borrow();
        let epoch = net.epochs.len() - 1;
        let items = net.epochs[epoch].1.clone();
        let frontier = self.frontier_of(&items);
        Context {
            net: self.clone(),
            mech: mech,
            items: items,
            frontier: frontier,
            initial_epoch: epoch,
            current_epoch: epoch,
        }
    }
    /// dot writes the network in the graphviz DOT language.
    fn dot<W>(&self, w: &mut W) -> ::std::io::Result<()>
        where W: Write
    {
        let net = self.net.borrow();
        let mut body = String::new();
        for id in 0..net.graph.len() {
            let label = format!("id={}  {}", id, &net.graph[id].data.as_str().clone());
            body.push_str(format!("  N{} [shape=box,label={:?}];\n", id, label).as_str());
        }
        body.pop();
        let mut edges = net.graph
            .iter()
            .flat_map(|item| {
                let id = item.id;
                item.adj.iter().map(move |&o| {
                    let mut v = vec![id, o];
                    v.sort();
                    (v[0], v[1])
                })
            })
            .collect::<Vec<_>>();
        edges.sort();
        edges.dedup();
        for (i, o) in edges {
            body.push_str(format!("\n  N{} -- N{};", i, o).as_str());
        }
        write!(w, "graph G {{\n{}\n}}\n", body)
    }
}

/// MechanismRegistry maintains a set of mechanisms used by the knowledge
/// network. A mechanism is a function which takes a Context and an
/// iteration number.
struct MechanismRegistry<'a> {
    reg: Vec<(&'static str, &'a Fn(Context, u64))>,
}

impl<'a> MechanismRegistry<'a> {
    fn new() -> MechanismRegistry<'a> {
        MechanismRegistry { reg: Vec::new() }
    }
    fn register(&mut self, name: &'static str, mech: &'a Fn(Context, u64)) {
        self.reg.push((name, mech));
    }
}

/// Skn maintains a knowledge network and the mechanisms interacting with it.
pub struct Skn<'a> {
    network: Network,
    reg: MechanismRegistry<'a>,
    t: u64,
}
impl<'a> fmt::Debug for Skn<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let net = self.network.net.borrow();
        write!(f, "Skn {{ net: {:?} }}", net)
    }
}

impl<'a> Skn<'a> {
    /// embryo is a non-empty collection of initial knowledge artifacts of
    /// the form (mechanism name, data), and iterations is the number of
    /// iterations to run each mechanism.
    pub fn new<U>(embryo: U, iterations: u64) -> Skn<'a>
        where U: IntoIterator<Item = (&'static str, String)>
    {
        Skn {
            network: Network::new(embryo),
            reg: MechanismRegistry::new(),
            t: iterations,
        }
    }
    /// register adds a new mechanism, given by its name and a function
    /// which takes a Context and an iteration number, for use with the
    /// knowledge network.
    pub fn register(&mut self, name: &'static str, mech: &'a Fn(Context, u64)) {
        self.reg.register(name, mech);
    }
    /// run calls each mechanism `iteration` number of times (set when this
    /// Skn was created) with a refreshed context on each iteration
    /// (according to the latest epoch of the knowledge network).
    pub fn run(&self) {
        for t in 1..self.t + 1 {
            for &(name, mech) in &self.reg.reg {
                let ctx = self.network.context(name);
                mech(ctx, t)
            }
        }
    }
    /// dot writes the network in the graphviz DOT language.
    pub fn dot<W>(&self, w: &mut W) -> ::std::io::Result<()>
        where W: Write
    {
        self.network.dot(w)
    }
}
