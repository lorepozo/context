extern crate rand;

use std::cmp::{min, max};
use std::collections::{BTreeMap, HashSet, HashMap};
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use rand::distributions::{IndependentSample, Range};

const CTX_MIN_SIZE: usize = 5;
const NET_MAX_SIZE: usize = 128;

#[derive(Debug)]
struct Item {
    mech: &'static str,
    data: Rc<String>,
    counts: BTreeMap<usize, u64>,
    adj: HashSet<usize>,
}

impl Item {
    fn new(mech: &'static str, adj: HashSet<usize>, data: String) -> Item {
        Item {
            mech: mech,
            data: Rc::new(data),
            counts: BTreeMap::new(),
            adj: adj,
        }
    }
    fn add_count(&mut self, epoch: usize, count: u64) {
        let prev_count: u64 = {
            *self.counts.get(&epoch).unwrap_or(&0)
        };
        self.counts.insert(epoch, count + prev_count);
    }
    fn recent_count(&self, epoch: usize) -> u64 {
        self.counts.iter().filter(|&(e, _)| *e >= epoch).fold(0, |s, (_, c)| s + c)
    }
}

pub struct Context {
    net: Network,
    mech: &'static str,
    items: HashSet<usize>,
    frontier: HashSet<usize>,
    initial_epoch: usize,
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
    /// graph, of the form (mechName, data). Must be non-empty.
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
            let mut net = network.net.borrow_mut();
            // clique of embryo as base
            let mut id = 0;
            let embryo: Vec<(&'static str, String)> = embryo.into_iter().collect();
            let edges: HashSet<usize> = (0..embryo.len()).collect();
            net.graph = embryo.into_iter()
                .map(|(mech, data)| {
                    let mut edges = edges.clone();
                    edges.remove(&id);
                    let item = Item::new(mech, edges, data);
                    id = id + 1;
                    item
                })
                .collect();
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
    fn item_count(&self, epoch: usize, id: usize, count: u64) {
        let mut net = self.net.borrow_mut();
        {
            let mut item = &mut net.graph[id];
            item.add_count(epoch, count);
        }
        net.epochs[epoch].2.insert(id);
    }
    fn ids_to_contents<U>(&self, items: U) -> Vec<(usize, &'static str, Rc<String>)>
        where U: IntoIterator<Item = usize>
    {
        let net = self.net.borrow();
        items.into_iter()
            .map(move |id| {
                let item = &net.graph[id];
                (id, item.mech, item.data.clone())
            })
            .collect()
    }
    fn orient(&self, epoch: usize, id: usize) {
        let mut net = self.net.borrow_mut();
        let mut ctx = HashSet::new();
        let n = net.graph.len();
        if n < net.context_min_size {
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

            // start with the given node. add its neighbors, select most
            // popular neighbor and repeat. stop when we get to `size` or
            // when there are no new neighbors.
            let mut selected = id;
            while ctx.len() < size {
                let ext: HashSet<usize> = net.graph[selected]
                    .adj
                    .difference(&ctx)
                    .cloned()
                    .collect();
                if ext.is_empty() {
                    break;
                }
                selected = ext.iter()
                    .max_by_key(|&id| {
                        let item = &net.graph[*id];
                        item.recent_count(epoch)
                    })
                    .unwrap()
                    .clone();
                ctx.extend(ext);
            }
        }
        net.epochs.push((id, ctx, HashSet::new()))
    }
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
                .flat_map(|&(_, _, ref ids)| ids.clone())
                .collect(); // removes duplicates
            let mut sum = 0;
            let mut counts: HashSet<(usize, u64)> = ids.iter()
                .map(|&id| {
                    let item = &net.graph[id];
                    let count = item.recent_count(epoch);
                    sum += count;
                    (id, count)
                })
                .collect();
            if sum == 0 {
                // uniform in context if no recent accesses
                counts = ids.iter().map(|&id| (id, 1)).collect();
                sum = counts.len() as u64
            }
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
            // update other end of new edges
            for oid in &edges {
                let ref mut item = net.graph[*oid];
                item.adj.insert(id);
            }

            let item = Item::new(mech, edges, data);
            net.graph.push(item);
        }
        self.orient(epoch, id);
        id
    }
    fn frontier_of(&self, items: &HashSet<usize>) -> HashSet<usize> {
        let net = self.net.borrow();
        let frontier: HashSet<usize> = items.iter()
            .flat_map(|&id| {
                let item = &net.graph[id];
                item.adj.clone()
            })
            .collect();
        frontier.difference(items).cloned().collect()
    }
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
}

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
    pub fn new<U>(embryo: U, iterations: u64) -> Skn<'a>
        where U: IntoIterator<Item = (&'static str, String)>
    {
        Skn {
            network: Network::new(embryo),
            reg: MechanismRegistry::new(),
            t: iterations,
        }
    }
    pub fn register(&mut self, name: &'static str, mech: &'a Fn(Context, u64)) {
        self.reg.register(name, mech);
    }
    pub fn run(&self) {
        for t in 1..self.t + 1 {
            for &(name, mech) in &self.reg.reg {
                let ctx = self.network.context(name);
                mech(ctx, t)
            }
        }
    }
}
