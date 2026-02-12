# c12-parser

![Crates.io Version](https://img.shields.io/crates/v/c12-parser)
![Crates.io Total Downloads](https://img.shields.io/crates/d/c12-parser)
![Crates.io License](https://img.shields.io/crates/l/c12-parser)

## Installation

Add this crate by cargo

```bash
cargo add c12-parser
```

## Usage

```rust
use c12_parser::{
    parse_json,
    stringify_json,
    FormatOptions,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let text = r#"
{
  "name": "c12-parser",
  "version": "1.0.0"
}
"#;

    // Parse JSON into a serde_json::Value
    let value = parse_json(text)?;

    // Mutate the config as needed
    let mut obj = value.as_object().cloned().unwrap();
    obj.insert("debug".into(), true.into());

    // Control how formatting is preserved
    let opts = FormatOptions {
        indent: None,               // auto-detect indent from original text
        preserve_indentation: true, // keep original indentation where possible
        preserve_whitespace: true,  // keep leading/trailing whitespace
        sample_size: 1024,
    };

    // Stringify back to JSON while preserving formatting
    let output = stringify_json(&obj.into(), Some(&opts))?;
    println!("{output}");

    Ok(())
}
```

## Contribution

<details>
  <summary>Local development</summary>

- Clone this repository
- Install the latest version of [Rust](https://rust-lang.org/)
- Run tests using `cargo test` or `cargo run`

</details>

## License

Published under the [MIT](./LICENSE) license.
Made by [@YONGQI](https://github.com/betterhyq) 💛
