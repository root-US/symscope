def parse_config(path):
    raw = read_file(path)
    return validate(raw)

def read_file(path):
    with open(path) as f:
        return f.read()

if __name__ == "__main__":
    cfg = parse_config("app.toml")
