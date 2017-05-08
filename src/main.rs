// This file is part of Context, a program and library for machine learning.
// This program is available at <https://docs.lucasem.com/context_src>
// Copyright (C) 2017  Lucas E. Morales <lucas@lucasem.com>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate clap;
extern crate rand;
extern crate regex;
extern crate tempdir;

pub mod knowledge;
pub mod ec;

use std::fs::File;
use clap::{Arg, App};

fn argparse() -> Option<String> {
    let matches = App::new("skn with ec")
        .arg(Arg::with_name("dot")
                 .long("dot")
                 .value_name("FILE")
                 .help("writes graphviz dot to file")
                 .takes_value(true))
        .get_matches();
    match matches.value_of("dot") {
        Some(s) => Some(String::from(s)),
        _ => None,
    }
}

fn main() {
    let dot = argparse();

    let t = ec::iter_max();
    let embryo = ec::embryo();
    let mech = ec::mech;
    let mut skn = knowledge::Skn::new(embryo, t);
    skn.register("ec", &mech);
    skn.run();
    if let Some(path) = dot {
        let mut f = File::create(path).expect("create dot file");
        skn.dot(&mut f).unwrap();
    }
}

#[cfg(test)]
mod tests {
    extern crate rand;

    use knowledge::{Context, Skn};
    use rand::distributions::{IndependentSample, Gamma};

    /// a very basic mechanism, great for understanding what a mechanism
    /// could look like.
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
            ctx.grow(format!("{}", i));
        }
    }

    #[test]
    fn it_works() {
        let t = 20;
        let embryo = vec![("basic_mech_name", String::from("some data"))];
        let mech = basic_mech;
        let mut skn = Skn::new(embryo, t);
        skn.register("basic_mech_name", &mech);
        skn.run();
    }
}
