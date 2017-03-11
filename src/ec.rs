extern crate rand;
extern crate serde_json;
extern crate tempdir;

use std::f64;
use std::str;
use std::collections::{HashSet, HashMap};
use std::env;
use std::rc::Rc;
use std::path::Path;
use std::process::Command;

use knowledge::Context;

// masks used at compile-time to determine what gets logged
const LOG_LEVEL: u8 = 5;

pub const ITER_MAX: u64 = 11;
const EC_GRAMMAR_INCLUDE_PROGS: bool = false;
const EC_ACCESS_FACTOR: f64 = 400f64;
const EC_MAX_IN_ARTIFACT: usize = 20;
static PRIMS_ARR: [&'static str; 31] = ["B",
                                        "C",
                                        "S",
                                        "K",
                                        "I",
                                        "empty",
                                        "upper",
                                        "lower",
                                        "cap",
                                        "+",
                                        "0",
                                        "+1",
                                        "-1",
                                        "wc",
                                        "cc",
                                        "string-of-int",
                                        "findchar",
                                        "<SPACE>",
                                        "<COMMA>",
                                        "<DOT>",
                                        "<AT>",
                                        "<LESS-THAN>",
                                        "<GREATER-THAN>",
                                        "string-of-char",
                                        "substr",
                                        "replace",
                                        "replace-substr-first",
                                        "replace-substr-all",
                                        "nth",
                                        "fnth",
                                        "feach"];

/// course is for loading inputs for use with ec.
mod course {
    extern crate serde_json;
    extern crate tempdir;

    use std::env;
    use std::fs::File;
    use std::io::{Read, Write};
    use std::path::Path;
    use tempdir::TempDir;

    use knowledge::Context;

    fn curriculum_path() -> String {
        if let Ok(val) = env::var("EC_CURRICULUM") {
            val
        } else if Path::new("./curriculum/ec").exists() {
            String::from("./curriculum/ec")
        } else {
            panic!("could not find ec curriculum")
        }
    }

    pub fn read_curriculum(name: String) -> String {
        let path = Path::new(curriculum_path().as_str()).join(name);
        let mut f = File::open(&path).expect("opening curriculum file");
        let mut s = String::new();
        f.read_to_string(&mut s).expect("reading curriculum file");
        s
    }

    #[derive(Serialize, Deserialize)]
    struct Problem {
        i: String,
        o: String,
    }

    #[derive(Serialize, Deserialize)]
    struct Task {
        problems: Vec<Problem>,
        name: String,
    }

    #[derive(Serialize, Deserialize)]
    struct Comb {
        expr: String,
    }

    #[derive(Serialize, Deserialize)]
    pub struct Course {
        tasks: Vec<Task>,
        grammar: Vec<Comb>,
    }
    impl Course {
        /// load the course file corresponding to a particular iteration.
        pub fn load(i: u64) -> Course {
            let s = read_curriculum(format!("course_{:02}.json", i));
            serde_json::from_str(s.as_str()).expect("parsing course file")
        }
        /// merge a given Course with the grammar of combinators given in the Context.
        pub fn merge(&mut self, ctx: &Context) {
            let raw_items = ctx.get()
                .into_iter()
                .filter(|&(_, mech, _)| mech == "ec")
                .map(|(_, _, d)| d);
            for raw_item in raw_items {
                let item: Vec<String> = serde_json::from_str(raw_item.as_str())
                    .expect("parse combinator from context");
                let mut grammar: Vec<Comb> = item.into_iter()
                    .map(|s| Comb { expr: s })
                    .collect();
                self.grammar.append(&mut grammar);
            }
        }
        /// save a Course to a temporary file
        pub fn save(&self, i: u64) -> (TempDir, String) {
            let tmp_dir = TempDir::new("ec").expect("make temp dir");
            let path = tmp_dir.path().join(format!("ec_input_{}", i));
            let mut f = File::create(&path).expect("create temp file");
            let ser = serde_json::to_string(self).expect("serialize ec input");
            write!(f, "{}", ser).expect("write ec input");
            let path = String::from(path.to_str().unwrap());
            (tmp_dir, path)
        }
    }
}
use self::course::{Course, read_curriculum};


/// results is for parsing output from ec.
mod results {
    extern crate serde_json;

    #[derive(Clone, Serialize, Deserialize)]
    pub struct Comb {
        pub expr: String,
        pub log_likelihood: f64,
    }

    #[derive(Clone, Serialize, Deserialize)]
    pub struct TaskResult {
        pub expr: String,
        pub log_probability: f64,
    }

    #[derive(Serialize, Deserialize)]
    pub struct Task {
        pub task: String,
        pub result: Option<TaskResult>,
    }

    #[derive(Serialize, Deserialize)]
    pub struct Results {
        pub grammar: Vec<Comb>,
        pub programs: Vec<Task>,
        pub log_bic: Option<f64>,
        pub hit_rate: u64,
    }
    impl Results {
        pub fn from_string(raw: String) -> Results {
            serde_json::from_str(raw.as_str()).expect(format!("parse ec output {}", raw).as_str())
        }
    }
}
use self::results::Results;


fn ec_bin() -> String {
    if let Ok(val) = env::var("EC") {
        val
    } else if Path::new("./ec").exists() {
        String::from("./ec")
    } else {
        String::from("ec") // hopefully it's in $PATH
    }
}

/// embryo returns the embryo (embryo.json in the curriculum/ec directory)
/// for use by the Skn that uses ec.
pub fn embryo() -> Vec<(&'static str, String)> {
    let s = read_curriculum(String::from("embryo.json"));
    vec![("ec", s)]
}

/// primitives returns the set of expressions that are primitive to ec.
fn primitives() -> HashSet<String> {
    PRIMS_ARR.iter().map(|&s| String::from(s)).collect()
}

/// run_ec is the lower-level function that produces the ec results for a
/// given context and course iteration.
fn run_ec(ctx: &Context, i: u64) -> Results {
    let mut c = Course::load(i);
    c.merge(ctx);
    let (tmp_dir, path) = c.save(i);
    let output = Command::new(ec_bin())
        .arg(path)
        .output()
        .expect("run ec");
    drop(tmp_dir); // we can delete the temporary directory after ec has run
    if !output.status.success() {
        let err = String::from_utf8(output.stderr).unwrap();
        panic!("ec failed in iteration {}: {}", i, err)
    }
    let raw_results = String::from_utf8(output.stdout).expect("read ec output");
    if LOG_LEVEL & 4 != 0 {
        println!("{}", raw_results);
    }
    Results::from_string(raw_results)
}

/// exprs_in_context takes a set of items in the context as given by
/// Context::get() or Context::explore() and returns the combinators
/// contained in those that are readable by ec.
fn exprs_in_context(ctx: Vec<(usize, &'static str, Rc<String>)>) -> HashMap<String, usize> {
    ctx.into_iter()
        .filter(|&(_, mech, _)| mech == "ec")
        .map(|(id, _, d)| {
            let item: Vec<String> = serde_json::from_str(d.as_str())
                .expect("parse combinators from context");
            (id, item)
        })
        .flat_map(|(id, item)| item.into_iter().map(move |expr| (expr, id)))
        .collect()
}

/// find_exprs_in_context takes a set of items in the context as given by
/// Context::get() or Context::explore() and a vector of combinators.
/// It returns a vector of the same size as exprs, with Some(id) if a match
/// was found or None otherwise.
fn find_exprs_in_context(ctx: Vec<(usize, &'static str, Rc<String>)>,
                         exprs: &Vec<&String>)
                         -> Vec<Option<usize>> {
    let exprs_in_ctx = exprs_in_context(ctx);
    exprs.iter()
        .map(|&e| match exprs_in_ctx.get(e) {
            Some(id) => Some(*id),
            None => None,
        })
        .collect()
}

/// find_expr_in_context is like find_exprs_in_context but for a single
/// combinator.
fn find_expr_in_context(ctx: Vec<(usize, &'static str, Rc<String>)>,
                        expr: String)
                        -> Option<usize> {
    find_exprs_in_context(ctx, &vec![&expr])[0]
}

/// mech is the ec mechanism as it should be registered/used by an Skn
/// object. It wraps running ec with updating item access counts and adding
/// a new item where appropriate.
pub fn mech(ctx: Context, i: u64) {
    // run ec
    let results = run_ec(&ctx, i);
    let failures: Vec<&String> = results.programs
        .iter()
        .filter(|p| p.result.is_none())
        .map(|p| &p.task)
        .collect();
    if LOG_LEVEL & 1 != 0 {
        println!("ec at iteration {} with got hit-rate {}/{}. failed: {:?}",
                 i,
                 results.hit_rate,
                 results.programs.len(),
                 failures);
    }
    if LOG_LEVEL & 2 != 0 {
        println!("   using ctx {:?}", exprs_in_context(ctx.get()));
    }

    // retrieve learned combs
    let prims = primitives();
    let mut learned: Vec<(String, f64)> = results.grammar
        .iter()
        .map(|c| (c.expr.clone(), c.log_likelihood))
        .filter(|c| !prims.contains(&c.0) && c.1.is_finite())
        .collect();
    if EC_GRAMMAR_INCLUDE_PROGS {
        learned.extend(results.programs
            .iter()
            .filter(|t| t.result.is_some())
            .map(|t| {
                let ref r = t.result;
                let r = r.clone().unwrap();
                (r.expr, r.log_probability)
            })
            .filter(|c| !prims.contains(&c.0) && c.1.is_finite()));
    }

    // early return if no useful results
    if learned.is_empty() {
        return;
    }

    // orient to most probable comb
    let mut ctx = ctx;
    let most_probable = learned.iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .unwrap()
        .0
        .clone();
    let result = find_expr_in_context(ctx.explore(), most_probable);
    if let Some(id) = result {
        if LOG_LEVEL & 2 != 0 {
            println!("   ctx.orient({})", id);
        }
        ctx.orient(id);
        ctx = ctx.update();
    }

    // make accesses ~ usage
    learned.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap()); // reversed sort
    let exprs = &learned.iter().map(|&(ref s, _)| s).collect();
    let findings = find_exprs_in_context(ctx.get(), exprs);
    let mut access_info: Vec<(&String, f64, usize)> = learned
        .iter()
        .zip(findings.into_iter())
        .filter(|&(_, o)| o.is_some())
        .map(|(&(ref s, p), o)| (s, p, o.unwrap())) // s, p, id
        .filter(|&(_, p, _)| p.is_finite())
        .collect();
    let least = access_info.iter().map(|&(_, p, _)| p).fold(f64::INFINITY, f64::min);
    let most = access_info.iter().map(|&(_, p, _)| p).fold(f64::NEG_INFINITY, f64::max);
    access_info = access_info
        .into_iter()
        .map(|(s, p, id)| (s, EC_ACCESS_FACTOR * (p-least)/(most-least), id)) // normalize
        .filter(|&(_, f, _)| f.is_finite())
        .collect();
    for comb in &access_info {
        ctx.add_item_count(comb.2, comb.1 as u64);
    }

    // add item with probable combs, excluding primitives and combs in context
    let exprs_in_ctx = exprs_in_context(ctx.explore());
    let new_combs: Vec<String> = learned // already sorted by prob
        .iter()
        .map(|&(ref s, _)| s)
        .filter(|&s| !prims.contains(s) && !exprs_in_ctx.contains_key(s))
        .take(EC_MAX_IN_ARTIFACT)
        .map(|s| s.clone())
        .collect();
    if !new_combs.is_empty() {
        ctx.grow(json!(new_combs).to_string());
        if LOG_LEVEL & 2 != 0 {
            println!("   ctx.grow({:?})", new_combs);
        }
    }
}
