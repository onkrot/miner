use crossbeam::crossbeam_channel::unbounded;
use crossbeam::scope;
use ring::digest::{digest, Context, Digest, SHA256};
use std::fmt::Write as FmtWrite;
use std::fs::File;
use std::io::{self, BufRead};
use std::time::Instant;
use std::io::Write as IoWrite;
use reqwest;

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

async fn get_feed() -> String {
    let body = reqwest::get("https://www.rust-lang.org")
    .await?
    .text()
    .await?;

    //resp.text().unwrap()
    body
}

fn write_tree(tree: &Vec<Vec<Digest>>) -> std::io::Result<()> {
    let mut file = File::create("output.txt").expect("kek");
    fn pprint_tree(level: usize, num: usize, prefix: String, last:bool, tree: &Vec<Vec<Digest>>, file: &mut File) -> std::io::Result<()> {
        let prefix_current = if last { "`- " } else { "|- " };

        write!(file, "{}{}0x", prefix, prefix_current)?;
        for &byte in tree[level][num].as_ref() {
            write!(file, "{:02X}", byte)?;
        }
        writeln!(file)?;

        let prefix_child =  if last { "   " } else { "|  " };
        let prefix = prefix + prefix_child;

        if level > 0 {
            for i in num..=num+1 {
                pprint_tree(level - 1, i,prefix.to_string(), i == num+1, tree, file).unwrap();
            }
        }
        Ok(())
    }

    pprint_tree(tree.len() - 1, 0, "".to_string(), true, tree, &mut file)
}

fn main() {
    let file = File::open("input.txt").unwrap();
    let lines: Vec<String> = io::BufReader::new(file)
        .lines()
        .map(|l| l.expect("no parse"))
        .collect();
    let line1: Vec<&str> = lines[0].split_whitespace().collect();
    let n: usize = line1[0].parse().unwrap();
    let level1: Vec<Digest> = lines
        .iter()
        .skip(1)
        .map(|line| digest(&SHA256, line.as_bytes()))
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
            tree[ln - 1].push(ctx.finish());
        }
        count = tree[ln - 1].len();
    }
    let root = tree.last().unwrap()[0].as_ref();
    let mut s = String::new();
    for &byte in root {
        write!(&mut s, "{:02X}", byte).expect("Unable to write");
    }
    println!("root hash {}  check {}", s, first_zero(root, n));

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
        let mut st = String::new();
        for &byte in curr.as_ref() {
            write!(&mut st, "{:02X}", byte).expect("Unable to write");
        }
        println!();
        //let t = nonce as f64 / now.elapsed().as_millis() as f64 * 1000.0;
        println!("{}", first_zero(curr.as_ref(), n));
        println!(
            "found hash {} took {}ms n:{}",
            st,
            now.elapsed().as_millis(),
            n
        );
        write_tree(&tree).unwrap();
        //write_tree(&tree);
    } else {
        println!("no solution");
    }
}
