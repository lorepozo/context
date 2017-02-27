extern crate rand;

pub mod knowledge;
use knowledge::{Skn, Context};

use rand::distributions::{IndependentSample, Gamma};

fn main() {
    let t = 20;
    let embryo = vec![("mech", vec![0, 1, 2, 3])];
    let mech = |ctx: Context, i| {
        let items = ctx.get();
        let front = ctx.explore();
        println!("looking at iteration {} with {} items in ctx and {} items in frontier",
                 i,
                 items.len(),
                 front.len() - items.len());
        // some random-ish accesses favoring older items
        let mut rng = rand::thread_rng();
        for (id, _, _) in items {
            let shape = 1f64 / (1f64 + id as f64);
            let gamma = Gamma::new(shape, 1f64);
            let cnt = 100f64 * gamma.ind_sample(&mut rng);
            ctx.add_item_count(id, cnt as u64);
        }
        // grow half the time
        if i % 2 == 0 {
            ctx.grow(vec![i as u8]);
        }
    };
    let mut skn = Skn::new(embryo, t);
    skn.register("test", &mech);
    skn.run();
}

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}
