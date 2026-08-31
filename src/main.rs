//! Constructor auto-mejorante.
//!
//! El binario lleva su genoma embebido. `evolve` muta el cerebro (una expresión
//! en el propio fuente), lo mide contra un evaluador congelado, y si hay
//! `--spawn` escribe un hijo que nace con esa versión.

mod dish;

use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command};
use std::time::{SystemTime, UNIX_EPOCH};

const GENERATION: u32 = 0;
const LINEAGE: &str = "0";
pub(crate) const BRAIN: &str = "0";

const GENOME: &[(&str, &str)] = &[
    ("Cargo.toml", include_str!("../Cargo.toml")),
    ("src/main.rs", include_str!("main.rs")),
    ("src/dish.rs", include_str!("dish.rs")),
    ("cell.svg", include_str!("../cell.svg")),
    ("README.md", include_str!("../README.md")),
    (".gitignore", include_str!("../.gitignore")),
    ("LICENSE", include_str!("../LICENSE")),
];

const MAX_DEPTH: usize = 7;
const DEFAULT_STEPS: u32 = 120;
const DEFAULT_LAMBDA: u32 = 30;
pub(crate) const XS: i32 = 5;

fn main() {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        None | Some("help") | Some("-h") | Some("--help") => help(),
        Some("identity") | Some("id") => {
            if let Err(e) = identity() {
                eprintln!("error: {e}");
                process::exit(1);
            }
        }
        Some("eval") => {
            let x = args.next();
            if let Err(e) = eval_cmd(x.as_deref()) {
                eprintln!("error: {e}");
                process::exit(1);
            }
        }
        Some("genome") => print_genome(),
        Some("evolve") => {
            if let Err(e) = evolve_cmd(args.collect()) {
                eprintln!("error: {e}");
                process::exit(1);
            }
        }
        Some("spawn") => {
            let dest = args.next().unwrap_or_else(|| {
                eprintln!("uso: mejorante spawn <directorio> [--build] [--force]");
                process::exit(2);
            });
            let mut force = false;
            let mut build = false;
            for flag in args {
                match flag.as_str() {
                    "--force" => force = true,
                    "--build" => build = true,
                    other => {
                        eprintln!("flag desconocida: {other}");
                        process::exit(2);
                    }
                }
            }
            if let Err(e) = spawn(Path::new(&dest), BRAIN, force, build) {
                eprintln!("error: {e}");
                process::exit(1);
            }
        }
        Some("dish") => {
            let mut steps = DEFAULT_STEPS;
            let mut lambda = DEFAULT_LAMBDA;
            let mut seed = entropy_seed();
            let mut delay_ms: u64 = 80;
            let rest: Vec<String> = args.collect();
            let mut it = rest.into_iter();
            while let Some(flag) = it.next() {
                match flag.as_str() {
                    "--steps" => {
                        steps = parse_u32(&next_val(&mut it, "--steps").unwrap_or_else(|e| {
                            eprintln!("error: {e}");
                            process::exit(2);
                        }))
                        .unwrap_or_else(|e| {
                            eprintln!("error: {e}");
                            process::exit(2);
                        });
                    }
                    "--lambda" => {
                        lambda = parse_u32(&next_val(&mut it, "--lambda").unwrap_or_else(|e| {
                            eprintln!("error: {e}");
                            process::exit(2);
                        }))
                        .unwrap_or_else(|e| {
                            eprintln!("error: {e}");
                            process::exit(2);
                        });
                    }
                    "--seed" => {
                        seed = parse_u64(&next_val(&mut it, "--seed").unwrap_or_else(|e| {
                            eprintln!("error: {e}");
                            process::exit(2);
                        }))
                        .unwrap_or_else(|e| {
                            eprintln!("error: {e}");
                            process::exit(2);
                        });
                    }
                    "--delay" => {
                        let v = next_val(&mut it, "--delay").unwrap_or_else(|e| {
                            eprintln!("error: {e}");
                            process::exit(2);
                        });
                        delay_ms = v.parse().unwrap_or_else(|_| {
                            eprintln!("no es un número: {v}");
                            process::exit(2);
                        });
                    }
                    other => {
                        eprintln!("flag desconocida: {other}");
                        process::exit(2);
                    }
                }
            }
            if let Err(e) = dish::run(steps, lambda, seed, delay_ms) {
                eprintln!("error: {e}");
                process::exit(1);
            }
        }
        Some(other) => {
            eprintln!("comando desconocido: {other}\n");
            help();
            process::exit(2);
        }
    }
}

fn help() {
    println!(
        "\
mejorante — constructor auto-mejorante (generación {GENERATION}, linaje {LINEAGE})
cerebro: {BRAIN}

  mejorante identity              generación, linaje, cerebro, sse
  mejorante eval [x]              cerebro vs objetivo
  mejorante evolve                busca un cerebro mejor
                   --steps N      default {DEFAULT_STEPS}
                   --lambda L     mutantes por paso, default {DEFAULT_LAMBDA}
                   --seed S       rng reproducible
                   --spawn <dir>  hijo con el campeón
                   --build        compila al hijo
                   --force        pisa un hijo anterior
                   --write        pisa src/main.rs de este proyecto
  mejorante dish                  anima una célula que se va ajustando
                   --steps N      default {DEFAULT_STEPS}
                   --lambda L     default {DEFAULT_LAMBDA}
                   --seed S       rng reproducible
                   --delay MS     ms entre frames (default 80)
  mejorante spawn <dir>           copia el genoma actual (sin buscar)
  mejorante genome                imprime las fuentes embebidas

El evaluador está congelado: f(x) = x² + 3x + 5 en x = -5..5.
No se copia por la red. Un solo hijo por corrida."
    );
}

fn identity() -> Result<(), Box<dyn Error>> {
    let expr = parse_expr(BRAIN)?;
    let err = sse(&expr);
    println!("mejorante");
    println!("generación  {GENERATION}");
    println!("linaje      {LINEAGE}");
    println!("cerebro     {BRAIN}");
    println!("sse         {err:.4}");
    println!("tamaño      {}", expr.size());
    println!("fitness     {:.4}", score(&expr));
    println!("archivos    {}", GENOME.len());
    println!();
    println!("muestra     x   cerebro   objetivo");
    for x in [0, 1, 2, 3, 5] {
        let xf = x as f64;
        println!(
            "          {x:>2}   {:>7.1}    {:>7.1}",
            expr.eval(xf),
            target(xf)
        );
    }
    if err == 0.0 {
        println!("\nóptimo: el cerebro reproduce el evaluador.");
    }
    Ok(())
}

fn eval_cmd(x_arg: Option<&str>) -> Result<(), Box<dyn Error>> {
    let expr = parse_expr(BRAIN)?;
    match x_arg {
        Some(s) => {
            let x: f64 = s.parse().map_err(|_| format!("no es un número: {s}"))?;
            println!(
                "x={x}  cerebro={}  objetivo={}  err={}",
                expr.eval(x),
                target(x),
                expr.eval(x) - target(x)
            );
        }
        None => {
            println!("  x   cerebro   objetivo     err²");
            for i in -XS..=XS {
                let x = i as f64;
                let y = expr.eval(x);
                let t = target(x);
                let d = y - t;
                println!("{i:>3}  {y:>8.1}  {t:>8.1}  {:>8.1}", d * d);
            }
            println!("sse  {:.4}", sse(&expr));
        }
    }
    Ok(())
}

fn print_genome() {
    for (i, (name, body)) in GENOME.iter().enumerate() {
        if i > 0 {
            println!();
        }
        println!("===== {name} =====");
        print!("{body}");
        if !body.ends_with('\n') {
            println!();
        }
    }
}

struct EvolveOpts {
    steps: u32,
    lambda: u32,
    seed: u64,
    spawn_dir: Option<PathBuf>,
    build: bool,
    force: bool,
    write: bool,
}

fn parse_evolve_opts(args: Vec<String>) -> Result<EvolveOpts, Box<dyn Error>> {
    let mut opts = EvolveOpts {
        steps: DEFAULT_STEPS,
        lambda: DEFAULT_LAMBDA,
        seed: entropy_seed(),
        spawn_dir: None,
        build: false,
        force: false,
        write: false,
    };
    let mut it = args.into_iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--steps" => opts.steps = parse_u32(&next_val(&mut it, "--steps")?)?,
            "--lambda" => opts.lambda = parse_u32(&next_val(&mut it, "--lambda")?)?,
            "--seed" => opts.seed = parse_u64(&next_val(&mut it, "--seed")?)?,
            "--spawn" => opts.spawn_dir = Some(PathBuf::from(next_val(&mut it, "--spawn")?)),
            "--build" => opts.build = true,
            "--force" => opts.force = true,
            "--write" => opts.write = true,
            other => return Err(format!("flag desconocida: {other}").into()),
        }
    }
    if opts.build && opts.spawn_dir.is_none() {
        return Err("--build pide --spawn <dir>".into());
    }
    Ok(opts)
}

fn next_val(it: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, Box<dyn Error>> {
    it.next().ok_or_else(|| format!("{flag} pide un valor").into())
}

fn parse_u32(s: &str) -> Result<u32, Box<dyn Error>> {
    s.parse().map_err(|_| format!("no es u32: {s}").into())
}

fn parse_u64(s: &str) -> Result<u64, Box<dyn Error>> {
    s.parse().map_err(|_| format!("no es u64: {s}").into())
}

fn entropy_seed() -> u64 {
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(1);
    t ^ ((process::id() as u64) << 32)
}

fn evolve_cmd(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let opts = parse_evolve_opts(args)?;
    let champ = evolve(opts.steps, opts.lambda, opts.seed)?;
    let improved = champ.sse < sse(&parse_expr(BRAIN)?) - 1e-9;

    println!();
    println!("seed       {}", opts.seed);
    println!("campeón    {}", champ.expr.emit());
    println!("sse        {:.4}", champ.sse);
    println!(
        "{}",
        if champ.sse == 0.0 {
            "resultado   óptimo"
        } else if improved {
            "resultado   mejoró"
        } else {
            "resultado   no mejoró"
        }
    );

    if opts.write {
        write_local_brain(&champ.expr.emit())?;
    }
    if let Some(dir) = opts.spawn_dir {
        spawn(&dir, &champ.expr.emit(), opts.force, opts.build)?;
    } else if !opts.write {
        println!("(nada escrito: usá --spawn <dir> o --write)");
    }
    Ok(())
}

fn evolve(steps: u32, lambda: u32, seed: u64) -> Result<Champion, Box<dyn Error>> {
    evolve_on(steps, lambda, seed, |step, best, improved| {
        if step == 0 || improved {
            let mark = if best.sse == 0.0 && step > 0 {
                "  óptimo"
            } else if improved {
                "  *"
            } else {
                ""
            };
            println!(
                "paso {step:>4}  sse {:>10.2}  {}{mark}",
                best.sse,
                best.expr.emit()
            );
        }
    })
}

pub(crate) fn evolve_on<F>(
    steps: u32,
    lambda: u32,
    seed: u64,
    mut hook: F,
) -> Result<Champion, Box<dyn Error>>
where
    F: FnMut(u32, &Champion, bool),
{
    let mut rng = Rng::new(seed);
    let expr = parse_expr(BRAIN)?;
    let mut best = Champion::from_expr(expr);
    hook(0, &best, false);
    for step in 1..=steps {
        let mut winner = best.clone();
        for _ in 0..lambda {
            let m = mutate(&best.expr, &mut rng);
            if m.depth() > MAX_DEPTH {
                continue;
            }
            let cand = Champion::from_expr(m);
            if cand.score < winner.score {
                winner = cand;
            }
        }
        if winner.score < best.score {
            best = winner;
            hook(step, &best, true);
        }
    }
    Ok(best)
}

fn write_local_brain(brain: &str) -> Result<(), Box<dyn Error>> {
    let main_path = Path::new("src/main.rs");
    if !looks_like_mejorante(Path::new(".")) {
        return Err("este directorio no parece un mejorante (corrés desde el crate)".into());
    }
    let src = fs::read_to_string(main_path)?;
    let next = patch_const_str(&src, "BRAIN", brain)?;
    fs::write(main_path, next)?;
    println!("escribió cerebro en {}", main_path.display());
    println!("compilá de nuevo para que el binario nazca con él");
    Ok(())
}

fn spawn(dest: &Path, brain: &str, force: bool, build: bool) -> Result<(), Box<dyn Error>> {
    let dest = normalize_dest(dest)?;
    assert_safe_dest(&dest)?;
    prepare_dest(&dest, force)?;

    let next_gen = GENERATION + 1;
    let next_lineage = format!("{LINEAGE}.{next_gen}");
    let child_main = rewrite_main(include_str!("main.rs"), next_gen, &next_lineage, brain)?;

    for (rel, contents) in GENOME {
        let body = if *rel == "src/main.rs" {
            child_main.clone()
        } else {
            (*contents).to_string()
        };
        let path = dest.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, body)?;
        println!("escribió {}", path.display());
    }

    println!(
        "hijo generación {next_gen} linaje {next_lineage} cerebro {brain} → {}",
        dest.display()
    );

    if build {
        let status = Command::new("cargo")
            .arg("build")
            .current_dir(&dest)
            .status()?;
        if !status.success() {
            return Err("cargo build del hijo falló".into());
        }
        println!("hijo compilado: {}/target/debug/mejorante", dest.display());
    }
    Ok(())
}

fn rewrite_main(
    src: &str,
    generation: u32,
    lineage: &str,
    brain: &str,
) -> Result<String, Box<dyn Error>> {
    let src = patch_const_u32(src, "GENERATION", GENERATION, generation)?;
    let src = patch_const_str(&src, "LINEAGE", lineage)?;
    patch_const_str(&src, "BRAIN", brain)
}

fn patch_const_u32(src: &str, name: &str, old: u32, new: u32) -> Result<String, Box<dyn Error>> {
    let from = format!("const {name}: u32 = {old};");
    let to = format!("const {name}: u32 = {new};");
    if !src.contains(&from) {
        return Err(format!("no encuentro {name} en el genoma").into());
    }
    Ok(src.replacen(&from, &to, 1))
}

fn patch_const_str(src: &str, name: &str, new_val: &str) -> Result<String, Box<dyn Error>> {
    if new_val.contains('"') || new_val.contains('\\') {
        return Err("el valor no puede tener comillas ni backslash".into());
    }
    let start_pat = format!("const {name}: &str = \"");
    let start = src
        .find(&start_pat)
        .ok_or_else(|| format!("no encuentro {name} en el genoma"))?;
    let value_start = start + start_pat.len();
    let rel_end = src[value_start..]
        .find('"')
        .ok_or_else(|| format!("const {name} sin cierre"))?;
    let value_end = value_start + rel_end;
    let mut out = String::with_capacity(src.len() + new_val.len());
    out.push_str(&src[..value_start]);
    out.push_str(new_val);
    out.push_str(&src[value_end..]);
    Ok(out)
}

fn normalize_dest(dest: &Path) -> Result<PathBuf, Box<dyn Error>> {
    if dest.as_os_str().is_empty() {
        return Err("directorio vacío".into());
    }
    if dest.is_absolute() {
        Ok(dest.to_path_buf())
    } else {
        Ok(env::current_dir()?.join(dest))
    }
}

fn assert_safe_dest(dest: &Path) -> Result<(), Box<dyn Error>> {
    let cwd = env::current_dir()?;
    if dest == cwd {
        return Err("no voy a sobreescribir el directorio actual".into());
    }
    let home = env::var_os("HOME").map(PathBuf::from);
    let forbidden = [
        PathBuf::from("/"),
        PathBuf::from("/usr"),
        PathBuf::from("/bin"),
        PathBuf::from("/sbin"),
        PathBuf::from("/etc"),
        PathBuf::from("/System"),
        PathBuf::from("/Library"),
        PathBuf::from("/Applications"),
        PathBuf::from("/private"),
    ];
    for p in &forbidden {
        if dest == p {
            return Err(format!("destino prohibido: {}", dest.display()).into());
        }
    }
    if let Some(home) = &home {
        if dest == home {
            return Err("no voy a escribir en $HOME".into());
        }
    }
    Ok(())
}

fn prepare_dest(dest: &Path, force: bool) -> Result<(), Box<dyn Error>> {
    if !dest.exists() {
        fs::create_dir_all(dest)?;
        return Ok(());
    }
    if dest.is_file() {
        return Err(format!("{} es un archivo", dest.display()).into());
    }
    let empty = dest.read_dir()?.next().is_none();
    if empty {
        return Ok(());
    }
    if !force {
        return Err(format!(
            "{} ya existe y no está vacío (usa --force si es un mejorante anterior)",
            dest.display()
        )
        .into());
    }
    if !looks_like_mejorante(dest) {
        return Err(format!("{} no parece un mejorante; no lo borro", dest.display()).into());
    }
    fs::remove_dir_all(dest)?;
    fs::create_dir_all(dest)?;
    Ok(())
}

fn looks_like_mejorante(dir: &Path) -> bool {
    let cargo = fs::read_to_string(dir.join("Cargo.toml")).unwrap_or_default();
    let main = fs::read_to_string(dir.join("src/main.rs")).unwrap_or_default();
    cargo.contains("name = \"mejorante\"")
        && main.contains("const GENERATION:")
        && main.contains("const BRAIN:")
}

pub(crate) fn target(x: f64) -> f64 {
    x * x + 3.0 * x + 5.0
}

pub(crate) fn sse(expr: &Expr) -> f64 {
    let mut s = 0.0;
    for i in -XS..=XS {
        let x = i as f64;
        let y = expr.eval(x);
        if !y.is_finite() {
            return f64::MAX;
        }
        let d = y - target(x);
        s += d * d;
    }
    s
}

fn score(expr: &Expr) -> f64 {
    let e = sse(expr);
    if e == f64::MAX {
        e
    } else {
        e + 0.01 * expr.size() as f64
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Expr {
    X,
    Const(i32),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
}

#[derive(Clone)]
pub(crate) struct Champion {
    pub(crate) expr: Expr,
    pub(crate) sse: f64,
    score: f64,
}

impl Champion {
    fn from_expr(expr: Expr) -> Self {
        let sse = sse(&expr);
        let score = if sse == f64::MAX {
            sse
        } else {
            sse + 0.01 * expr.size() as f64
        };
        Self { expr, sse, score }
    }
}

impl Expr {
    pub(crate) fn eval(&self, x: f64) -> f64 {
        match self {
            Expr::X => x,
            Expr::Const(c) => *c as f64,
            Expr::Add(a, b) => a.eval(x) + b.eval(x),
            Expr::Sub(a, b) => a.eval(x) - b.eval(x),
            Expr::Mul(a, b) => a.eval(x) * b.eval(x),
        }
    }

    fn size(&self) -> usize {
        match self {
            Expr::X | Expr::Const(_) => 1,
            Expr::Add(a, b) | Expr::Sub(a, b) | Expr::Mul(a, b) => 1 + a.size() + b.size(),
        }
    }

    fn depth(&self) -> usize {
        match self {
            Expr::X | Expr::Const(_) => 1,
            Expr::Add(a, b) | Expr::Sub(a, b) | Expr::Mul(a, b) => 1 + a.depth().max(b.depth()),
        }
    }

    pub(crate) fn emit(&self) -> String {
        match self {
            Expr::X => "x".into(),
            Expr::Const(c) => c.to_string(),
            Expr::Add(a, b) => format!("(+ {} {})", a.emit(), b.emit()),
            Expr::Sub(a, b) => format!("(- {} {})", a.emit(), b.emit()),
            Expr::Mul(a, b) => format!("(* {} {})", a.emit(), b.emit()),
        }
    }
}

struct Parser<'a> {
    s: &'a [u8],
    i: usize,
}

pub(crate) fn parse_expr(src: &str) -> Result<Expr, Box<dyn Error>> {
    let mut p = Parser {
        s: src.as_bytes(),
        i: 0,
    };
    let e = p.parse()?;
    p.skip();
    if p.i != p.s.len() {
        return Err("sobraron tokens en el cerebro".into());
    }
    Ok(e)
}

impl Parser<'_> {
    fn skip(&mut self) {
        while self.i < self.s.len() && self.s[self.i].is_ascii_whitespace() {
            self.i += 1;
        }
    }

    fn parse(&mut self) -> Result<Expr, Box<dyn Error>> {
        self.skip();
        if self.i >= self.s.len() {
            return Err("expresión vacía".into());
        }
        match self.s[self.i] {
            b'x' | b'X' => {
                self.i += 1;
                Ok(Expr::X)
            }
            b'(' => {
                self.i += 1;
                self.skip();
                if self.i >= self.s.len() {
                    return Err("falta operador".into());
                }
                let op = self.s[self.i];
                self.i += 1;
                let a = self.parse()?;
                let b = self.parse()?;
                self.skip();
                if self.i >= self.s.len() || self.s[self.i] != b')' {
                    return Err("falta )".into());
                }
                self.i += 1;
                match op {
                    b'+' => Ok(Expr::Add(Box::new(a), Box::new(b))),
                    b'-' => Ok(Expr::Sub(Box::new(a), Box::new(b))),
                    b'*' => Ok(Expr::Mul(Box::new(a), Box::new(b))),
                    _ => Err(format!("operador '{}'", op as char).into()),
                }
            }
            b'-' | b'0'..=b'9' => {
                let start = self.i;
                if self.s[self.i] == b'-' {
                    self.i += 1;
                }
                if self.i >= self.s.len() || !self.s[self.i].is_ascii_digit() {
                    return Err("número inválido".into());
                }
                while self.i < self.s.len() && self.s[self.i].is_ascii_digit() {
                    self.i += 1;
                }
                let n: i32 = std::str::from_utf8(&self.s[start..self.i])
                    .map_err(|_| "utf8")?
                    .parse()
                    .map_err(|_| "número inválido")?;
                Ok(Expr::Const(n))
            }
            c => Err(format!("token '{}'", c as char).into()),
        }
    }
}

struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed | 1)
    }

    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }

    fn frac(&mut self) -> f64 {
        (self.next() >> 11) as f64 / ((1u64 << 53) as f64)
    }

    fn usize(&mut self, n: usize) -> usize {
        if n == 0 {
            0
        } else {
            (self.next() as usize) % n
        }
    }

    fn i32(&mut self, lo: i32, hi: i32) -> i32 {
        lo + self.usize((hi - lo + 1) as usize) as i32
    }
}

fn random_leaf(rng: &mut Rng) -> Expr {
    if rng.frac() < 0.5 {
        Expr::X
    } else {
        Expr::Const(rng.i32(-5, 5))
    }
}

fn random_op(rng: &mut Rng) -> u8 {
    match rng.usize(5) {
        0 | 1 => b'+',
        2 | 3 => b'*',
        _ => b'-',
    }
}

fn bin(op: u8, a: Expr, b: Expr) -> Expr {
    match op {
        b'+' => Expr::Add(Box::new(a), Box::new(b)),
        b'-' => Expr::Sub(Box::new(a), Box::new(b)),
        _ => Expr::Mul(Box::new(a), Box::new(b)),
    }
}

fn random_tree(rng: &mut Rng, depth: usize) -> Expr {
    if depth == 0 || rng.frac() < 0.45 {
        random_leaf(rng)
    } else {
        bin(
            random_op(rng),
            random_tree(rng, depth - 1),
            random_tree(rng, depth - 1),
        )
    }
}

fn replace_at<F>(expr: &Expr, at: usize, f: &mut F) -> Expr
where
    F: FnMut(&Expr) -> Expr,
{
    fn rec<F>(expr: &Expr, at: usize, offset: usize, f: &mut F) -> Expr
    where
        F: FnMut(&Expr) -> Expr,
    {
        if offset == at {
            return f(expr);
        }
        match expr {
            Expr::X | Expr::Const(_) => expr.clone(),
            Expr::Add(a, b) => rec_bin(expr, a, b, at, offset, f, b'+'),
            Expr::Sub(a, b) => rec_bin(expr, a, b, at, offset, f, b'-'),
            Expr::Mul(a, b) => rec_bin(expr, a, b, at, offset, f, b'*'),
        }
    }

    fn rec_bin<F>(
        expr: &Expr,
        a: &Expr,
        b: &Expr,
        at: usize,
        offset: usize,
        f: &mut F,
        op: u8,
    ) -> Expr
    where
        F: FnMut(&Expr) -> Expr,
    {
        let left_off = offset + 1;
        let right_off = left_off + a.size();
        if at < right_off {
            bin(op, rec(a, at, left_off, f), b.clone())
        } else if at < right_off + b.size() {
            bin(op, a.clone(), rec(b, at, right_off, f))
        } else {
            expr.clone()
        }
    }

    rec(expr, at, 0, f)
}

fn mutate(expr: &Expr, rng: &mut Rng) -> Expr {
    let n = expr.size();
    let at = rng.usize(n);
    replace_at(expr, at, &mut |node| {
        let r = rng.frac();
        if r < 0.25 {
            random_leaf(rng)
        } else if r < 0.45 {
            match node {
                Expr::Add(a, b) => bin(random_op(rng), *a.clone(), *b.clone()),
                Expr::Sub(a, b) => bin(random_op(rng), *a.clone(), *b.clone()),
                Expr::Mul(a, b) => bin(random_op(rng), *a.clone(), *b.clone()),
                _ => random_leaf(rng),
            }
        } else if r < 0.7 {
            if node.depth() >= MAX_DEPTH {
                node.clone()
            } else if rng.frac() < 0.5 {
                bin(random_op(rng), node.clone(), random_leaf(rng))
            } else {
                bin(random_op(rng), random_leaf(rng), node.clone())
            }
        } else if r < 0.9 {
            match node {
                Expr::Add(a, b) | Expr::Sub(a, b) | Expr::Mul(a, b) => {
                    if rng.frac() < 0.5 {
                        *a.clone()
                    } else {
                        *b.clone()
                    }
                }
                _ => random_leaf(rng),
            }
        } else {
            random_tree(rng, 2)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn genome_lists_the_project() {
        let names: Vec<_> = GENOME.iter().map(|(n, _)| *n).collect();
        assert!(names.contains(&"src/main.rs"));
        assert!(names.contains(&"src/dish.rs"));
        assert!(names.contains(&"cell.svg"));
        assert!(names.contains(&"Cargo.toml"));
        assert!(names.contains(&"README.md"));
    }

    #[test]
    fn parse_roundtrip() {
        let src = "(+ (* x x) (+ (* 3 x) 5))";
        let e = parse_expr(src).unwrap();
        assert_eq!(e.emit(), src);
        assert_eq!(e.eval(2.0), 15.0);
    }

    #[test]
    fn target_values() {
        assert_eq!(target(0.0), 5.0);
        assert_eq!(target(1.0), 9.0);
        assert_eq!(target(2.0), 15.0);
    }

    #[test]
    fn sse_of_perfect_brain_is_zero() {
        let e = parse_expr("(+ (* x x) (+ (* 3 x) 5))").unwrap();
        assert_eq!(sse(&e), 0.0);
    }

    #[test]
    fn sse_of_zero_is_bad() {
        let e = parse_expr("0").unwrap();
        assert!(sse(&e) > 1000.0);
    }

    #[test]
    fn rewrite_patches_brain_and_generation() {
        let next = rewrite_main(include_str!("main.rs"), 1, "0.1", "(+ x 1)").unwrap();
        assert!(next.contains("const GENERATION: u32 = 1;"));
        assert!(next.contains("const LINEAGE: &str = \"0.1\";"));
        assert!(next.contains("const BRAIN: &str = \"(+ x 1)\";"));
    }

    #[test]
    fn spawn_writes_child_with_brain() {
        let dir = env::temp_dir().join(format!("mejorante-test-{}", process::id()));
        let _ = fs::remove_dir_all(&dir);
        spawn(&dir, "(+ x 1)", false, false).unwrap();
        let child = fs::read_to_string(dir.join("src/main.rs")).unwrap();
        assert!(child.contains("const BRAIN: &str = \"(+ x 1)\";"));
        assert!(child.contains("const GENERATION: u32 = 1;"));
        assert!(dir.join("src/dish.rs").exists());
        assert!(dir.join("cell.svg").exists());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn refuses_to_spawn_over_cwd() {
        let cwd = env::current_dir().unwrap();
        let err = spawn(&cwd, BRAIN, true, false).unwrap_err();
        assert!(err.to_string().contains("directorio actual"));
    }

    #[test]
    fn evolve_improves_from_zero() {
        let champ = evolve(80, 25, 1).unwrap();
        let start = sse(&parse_expr(BRAIN).unwrap());
        assert!(champ.sse < start);
    }
}
