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

pub const ITER_MAX: u64 = 10;
const EC_GRAMMAR_INCLUDE_PROGS: bool = true;
const EC_ACCESS_FACTOR: f64 = 400f64;
const EC_MAX_IN_ARTIFACT: usize = 20;
static PRIMS_ARR: [&'static str; 26] = ["' '",
                                        "','",
                                        "'.'",
                                        "'<'",
                                        "'>'",
                                        "'@'",
                                        "+",
                                        "+1",
                                        "-1",
                                        "0",
                                        "B",
                                        "C",
                                        "I",
                                        "K",
                                        "S",
                                        "cap",
                                        "feach",
                                        "findchar",
                                        "fnth",
                                        "len",
                                        "lower",
                                        "nth",
                                        "string-of-char",
                                        "substr",
                                        "uncap",
                                        "upper"];
static EMBRYO: &'static str = r#"[]"#;

fn ec_bin() -> String {
    if let Ok(val) = env::var("EC") {
        val
    } else if Path::new("./ec").exists() {
        String::from("./ec")
    } else {
        String::from("ec") // hopefully it's in $PATH
    }
}

pub fn embryo() -> Vec<(&'static str, String)> {
    vec![("ec", String::from(EMBRYO))]
}
fn primitives() -> HashSet<String> {
    PRIMS_ARR.iter().map(|&s| String::from(s)).collect()
}


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
        pub fn load(i: u64) -> Course {
            let path = Path::new(curriculum_path().as_str()).join(format!("course_{:02}.json", i));
            let mut f = File::open(&path).expect("opening course file");
            let mut s = String::new();
            f.read_to_string(&mut s).expect("reading course file");
            serde_json::from_str(&s).expect("parsing course file")
        }
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
use self::course::Course;


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


fn run_ec(ctx: &Context, i: u64) -> Results {
    let mut c = Course::load(i);
    c.merge(ctx);
    let (tmp_dir, path) = c.save(i);
    let output = Command::new(ec_bin())
        .arg(path)
        .output()
        .expect("run ec");
    drop(tmp_dir);
    if !output.status.success() {
        let err = String::from_utf8(output.stderr).unwrap();
        panic!("ec failed in iteration {}: {}", i, err)
    }
    let raw_results = String::from_utf8(output.stdout).expect("read ec output");
    Results::from_string(raw_results)
}

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

fn find_expr_in_context(ctx: Vec<(usize, &'static str, Rc<String>)>,
                        expr: String)
                        -> Option<usize> {
    find_exprs_in_context(ctx, &vec![&expr])[0]
}

pub fn mech(ctx: Context, i: u64) {
    // run ec
    let results = run_ec(&ctx, i);
    println!("ec at iteration {} got hit-rate {}/{}",
             i,
             results.hit_rate,
             results.programs.len());
    // retrieve learned combs
    let mut learned: Vec<(String, f64)> = results.grammar
        .iter()
        .map(|c| (c.expr.clone(), c.log_likelihood))
        .collect();
    if EC_GRAMMAR_INCLUDE_PROGS {
        learned.extend(results.programs
            .iter()
            .filter(|t| t.result.is_some())
            .map(|t| {
                let r = &t.result;
                let r = r.clone().unwrap();
                (r.expr, r.log_probability)
            }));
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
    debug_assert!(access_info.is_empty());
    let least = access_info.iter().map(|&(_, p, _)| p).fold(f64::INFINITY, f64::min);
    let most = access_info.iter().map(|&(_, p, _)| p).fold(f64::NEG_INFINITY, f64::max);
    access_info = access_info
        .into_iter()
        .map(|(s, p, id)| (s, EC_ACCESS_FACTOR * (p-least)/(most-least), id)) // normalize
        .collect();
    for comb in &access_info {
        ctx.add_item_count(comb.2, comb.1 as u64);
    }
    // get probable combs, exclude primitives and combs in context
    let prims = primitives();
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
    }
}
