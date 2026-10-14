// Test double: first argument is an output file, the rest are recorded one per line.
fn main() {
    let mut args = std::env::args().skip(1);
    let out = args.next().expect("output path");
    let rest: Vec<String> = args.collect();
    let tmp = format!("{out}.tmp");
    std::fs::write(&tmp, rest.join("\n")).unwrap();
    std::fs::rename(tmp, out).unwrap();
}
