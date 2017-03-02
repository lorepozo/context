#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate rand;
extern crate tempdir;

pub mod knowledge;
pub mod ec;

fn main() {
    let t = ec::ITER_MAX;
    let embryo = ec::embryo();
    let mech = ec::mech;
    let mut skn = knowledge::Skn::new(embryo, t);
    skn.register("ec", &mech);
    skn.run();
    println!("{:?}", skn)
}

#[cfg(test)]
mod tests {
    extern crate rand;

    use knowledge::{Context, Skn};
    use rand::distributions::{IndependentSample, Gamma};

    fn basic_mech(ctx: Context, i: u64) {
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
    }

    #[test]
    fn it_works() {
        let t = 20;
        let embryo = vec![("mech", vec![0, 1, 2, 3])];
        let mech = basic_mech;
        let mut skn = Skn::new(embryo, t);
        skn.register("test", &mech);
        skn.run();
    }
}
