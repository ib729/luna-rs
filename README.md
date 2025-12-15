# Luna-RS

Rustified Luna

## Features

- **Lua script conversion** - For TI-Nspire OS 3.0.2+
- **Python script conversion** - For TI-Nspire CX II OS 5.2+
- **Plain text with LaTeX math** - Automatically converts LaTeX notation to Unicode/ASCII for display on calculator

## Usage

```bash
luna-rs <input> <output.tns>
```

### Examples

```bash
# Convert Lua script
luna-rs script.lua output.tns

# Convert Python script
luna-rs program.py output.tns

# Convert plain text with math notation
luna-rs notes.txt notes.tns
```

## LaTeX Math Support

When converting `.txt` files, Luna-RS automatically converts LaTeX-style math notation:

**Supported conversions:**
- Greek letters (lowercase): `\alpha`, `\beta`, `\gamma` → α, β, γ
- Greek letters (uppercase): `\Delta`, `\Sigma`, `\Omega` → Delta, Sigma, Omega (ASCII for compatibility)
- Superscripts: `x^2`, `x^{10}` → x², x¹⁰
- Subscripts: `H_2O`, `x_1` → H₂O, x₁
- Operators: `\pm`, `\times`, `\div`, `\leq`, `\geq` → ±, ×, ÷, ≤, ≥
- Arrows: `\to`, `\rightarrow` → `->`
- Big operators: `\sum`, `\int` → `SUM`, `INT` (ASCII for compatibility)
- Symbols: `\infty` → `inf`

**TI-Nspire Compatibility:**
Due to font limitations on the TI-Nspire, some symbols use ASCII equivalents to avoid blank rectangles:
- Uppercase Greek letters (Δ, Σ, Ω, etc.) use ASCII (Delta, Sigma, Omega)
- Arrows, quantifiers (∀, ∃), and big operators (∑, ∫, ∂, ∇) use ASCII
- Lowercase Greek letters, superscripts, subscripts, and basic operators use Unicode
- Subscripts/superscripts that can't be converted use parentheses: `\int_a^b` → `INT(a)(b)`
  - Unicode only has superscripts for: 0-9, +, -, =, (, ), n, i, x, y
  - Missing superscript letters (a, b, c, etc.) are shown in parentheses to avoid blank `^` characters

## Building

Requirements:
- Rust 1.70+

```bash
cd luna-rs
cargo build --release
```

The binary will be at `target/release/luna-rs`

## Testing

```bash
cargo test
```

## License

- Upstream Luna: Mozilla Public License v1.1 (MPL-1.1)
- This repository contains a mix of:
  - MPL-1.1 licensed files derived from upstream Luna
  - MIT licensed new Rust code

See `NOTICE`, `LICENSE.MPL`, and `LICENSE.MIT` for details.

## Credits

Based on the original [Luna](https://github.com/ndless-nspire/Luna)

TI-Nspire is a trademark of Texas Instruments. This project is not affiliated with or endorsed by Texas Instruments.
