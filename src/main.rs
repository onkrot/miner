use crossbeam::crossbeam_channel::unbounded;
use crossbeam::scope;
use data_encoding::HEXLOWER;
use reqwest;
use ring::digest::{Context, SHA256};
use serde_json::{Result, Value};
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::time::Instant;

static URL: &str = "https://blockchain.info/unconfirmed-transactions?format=json";

fn first_zero(x: &[u8], n: usize) -> bool {
    let k = n / 8;
    for i in 0..k {
        if x[i] != 0 {
            return false;
        }
    }

    let mut t = (n % 8) as u8;
    if t > 0 {
        let mut mask: u8 = 0;
        while t > 0 {
            mask |= 1 << (8 - t);
            t -= 1;
        }
        x[k] & mask == 0
    } else {
        true
    }
}

fn get_transactions() -> std::result::Result<String, reqwest::Error> {
    let body = reqwest::blocking::get(URL)?.text()?;

    Ok(body)
}

fn get_hashes(data: &str) -> Result<Vec<String>> {
    let v: Value = serde_json::from_str(data)?;
    let hashes: Vec<String> = v["txs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tx| tx.get("hash").unwrap().as_str().unwrap().to_string())
        .collect();

    Ok(hashes)
}

fn write_tree(tree: &Vec<Vec<Vec<u8>>>) -> std::io::Result<()> {
    let mut file = File::create("output.txt").expect("kek");
    fn pprint_tree(
        level: usize,
        num: usize,
        prefix: String,
        last: bool,
        tree: &Vec<Vec<Vec<u8>>>,
        file: &mut File,
    ) -> std::io::Result<()> {
        let prefix_current = if last { "`- " } else { "|- " };
        if num >= tree[level].len() {
            return Ok(());
        }
        writeln!(
            file,
            "{}{}0x{}",
            prefix,
            prefix_current,
            HEXLOWER.encode(&tree[level][num].as_ref())
        )?;

        let prefix_child = if last { "   " } else { "|  " };
        let prefix = prefix + prefix_child;

        if level > 0 {
            let t = num * 2;
            for i in t..=t + 1 {
                pprint_tree(level - 1, i, prefix.to_string(), i == t + 1, tree, file).unwrap();
            }
        }
        Ok(())
    }
    pprint_tree(tree.len() - 1, 0, "".to_string(), true, tree, &mut file)
}

#[allow(dead_code)]
fn read_tx_from_file<P: AsRef<Path>>(path: P) -> Vec<Vec<u8>> {
    let file = File::open(path).unwrap();
    let lines: Vec<String> = io::BufReader::new(file)
        .lines()
        .map(|l| l.expect("no parse"))
        .collect();
    lines
        .iter()
        .skip(1)
        .map(|line| HEXLOWER.decode(line.as_bytes()).unwrap())
        .collect()
}

fn main() {
    let file = File::open("input.txt").unwrap();
    let lines: Vec<String> = io::BufReader::new(file)
        .lines()
        .map(|l| l.expect("no parse"))
        .collect();
    let line1: Vec<&str> = lines[0].split_whitespace().collect();
    let n: usize = line1[0].parse().unwrap();

    let level1: Vec<Vec<u8>> = get_hashes(get_transactions().unwrap().as_str())
        .unwrap()
        .iter()
        .map(|st| HEXLOWER.decode(st.as_bytes()).unwrap())
        .collect();

    let mut tree = Vec::new();
    tree.push(level1);
    let mut count = tree[0].len();
    while count > 1 {
        let l = tree.len() - 1;
        if tree[l].len() % 2 != 0 {
            let t = tree[l].last().unwrap().clone();
            tree[l].push(t);
        }
        tree.push(Vec::new());
        let ln = tree.len();
        for i in (0..tree[l].len()).step_by(2) {
            let mut ctx = Context::new(&SHA256);
            ctx.update(tree[l][i].as_ref());
            ctx.update(tree[l][i + 1].as_ref());
            tree[ln - 1].push(ctx.finish().as_ref().to_vec());
        }
        count = tree[ln - 1].len();
    }
    let root = tree.last().unwrap()[0].as_ref();

    println!(
        "root hash 0x{}  check {}",
        HEXLOWER.encode(root),
        first_zero(root, n)
    );

    let now = Instant::now();
    let (s, r) = unbounded();
    let num_threads = 8;
    let num_work = std::u32::MAX / num_threads;
    let _ = scope(|scope| {
        for i in 0..num_threads {
            let (s1, r1) = (s.clone(), r.clone());
            scope.spawn(move |_| {
                let mut nonce: u32 = i * num_work;
                let mut ctx = Context::new(&SHA256);
                ctx.update(root);
                ctx.update(&nonce.to_le_bytes());
                let mut curr = ctx.finish();
                while (nonce < ((i + 1) * num_work)) && r1.is_empty() {
                    if first_zero(curr.as_ref(), n) {
                        println!("break {}", nonce);
                        s1.send(nonce).unwrap();
                        break;
                    }
                    nonce += 1;
                    ctx = Context::new(&SHA256);
                    ctx.update(root);
                    ctx.update(&nonce.to_le_bytes());
                    curr = ctx.finish();
                }
                drop(s1);
                println!("thread {} stoped on {}", i, nonce);
            });
        }
    });
    drop(s);

    if let Ok(nonce) = r.recv() {
        let mut ctx = Context::new(&SHA256);
        ctx.update(root);
        ctx.update(&nonce.to_le_bytes());
        let curr = ctx.finish();
        println!("nonce {}", nonce);
        println!();
        //let t = nonce as f64 / now.elapsed().as_millis() as f64 * 1000.0;
        println!("{}", first_zero(curr.as_ref(), n));
        println!(
            "found hash 0x{} took {}ms n:{}",
            HEXLOWER.encode(curr.as_ref()),
            now.elapsed().as_millis(),
            n
        );
        write_tree(&tree).unwrap();
    } else {
        println!("no solution");
    }
}
