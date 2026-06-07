fn parse_config(path: &str) -> Config {
    let raw = read_file(path);
    validate(raw)
}

fn read_file(path: &str) -> String {
    std::fs::read_to_string(path).unwrap()
}

fn main() {
    let cfg = parse_config("app.toml");
    println!("{:?}", cfg);
    // parse_config appears in this comment but must NOT count as a call
}
