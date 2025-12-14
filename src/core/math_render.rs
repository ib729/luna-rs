// MIT License - New code for Luna-RS
// See LICENSE.MIT for full license text

//! Simple LaTeX-to-Unicode converter for TI-Nspire
//!
//! Converts common LaTeX math notation to Unicode characters that
//! render correctly on TI-Nspire calculators.

use std::collections::HashMap;
use std::sync::LazyLock;

/// LaTeX command to Unicode mapping
static LATEX_MAP: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();

    // Greek lowercase
    m.insert("\\alpha", "α");
    m.insert("\\beta", "β");
    m.insert("\\gamma", "γ");
    m.insert("\\delta", "δ");
    m.insert("\\epsilon", "ε");
    m.insert("\\varepsilon", "ε");
    m.insert("\\zeta", "ζ");
    m.insert("\\eta", "η");
    m.insert("\\theta", "θ");
    m.insert("\\vartheta", "ϑ");
    m.insert("\\iota", "ι");
    m.insert("\\kappa", "κ");
    m.insert("\\lambda", "λ");
    m.insert("\\mu", "μ");
    m.insert("\\nu", "ν");
    m.insert("\\xi", "ξ");
    m.insert("\\pi", "π");
    m.insert("\\varpi", "ϖ");
    m.insert("\\rho", "ρ");
    m.insert("\\varrho", "ϱ");
    m.insert("\\sigma", "σ");
    m.insert("\\varsigma", "ς");
    m.insert("\\tau", "τ");
    m.insert("\\upsilon", "υ");
    m.insert("\\phi", "φ");
    m.insert("\\varphi", "ϕ");
    m.insert("\\chi", "χ");
    m.insert("\\psi", "ψ");
    m.insert("\\omega", "ω");

    // Greek uppercase - use Latin letters for TI-Nspire compatibility
    // The TI-Nspire font doesn't reliably support uppercase Greek
    m.insert("\\Gamma", "Gamma");
    m.insert("\\Delta", "Delta");
    m.insert("\\Theta", "Theta");
    m.insert("\\Lambda", "Lambda");
    m.insert("\\Xi", "Xi");
    m.insert("\\Pi", "Pi");
    m.insert("\\Sigma", "Sigma");
    m.insert("\\Upsilon", "Upsilon");
    m.insert("\\Phi", "Phi");
    m.insert("\\Psi", "Psi");
    m.insert("\\Omega", "Omega");

    // Math operators
    m.insert("\\times", "×");
    m.insert("\\div", "÷");
    m.insert("\\cdot", "·");
    m.insert("\\pm", "±");
    m.insert("\\mp", "∓");
    m.insert("\\ast", "∗");
    m.insert("\\star", "⋆");
    m.insert("\\circ", "∘");
    m.insert("\\bullet", "•");

    // Relations - keep Unicode for these as they work on TI-Nspire
    m.insert("\\leq", "≤");
    m.insert("\\le", "≤");
    m.insert("\\geq", "≥");
    m.insert("\\ge", "≥");
    m.insert("\\neq", "≠");
    m.insert("\\ne", "≠");
    m.insert("\\approx", "≈");
    m.insert("\\equiv", "≡");
    m.insert("\\sim", "∼");
    m.insert("\\simeq", "≃");
    m.insert("\\cong", "≅");
    m.insert("\\propto", "∝");
    m.insert("\\ll", "≪");
    m.insert("\\gg", "≫");
    // Set membership - use ASCII as these may not render
    m.insert("\\subset", "<");
    m.insert("\\supset", ">");
    m.insert("\\subseteq", "<=");
    m.insert("\\supseteq", ">=");
    m.insert("\\in", "in");
    m.insert("\\notin", "not in");
    m.insert("\\ni", "ni");
    m.insert("\\perp", "_|_");
    m.insert("\\parallel", "||");

    // Arrows - use ASCII fallbacks for TI-Nspire compatibility
    m.insert("\\leftarrow", "<-");
    m.insert("\\rightarrow", "->");
    m.insert("\\to", "->");  // Common in limits: lim_{x \to \infty}
    m.insert("\\uparrow", "^");
    m.insert("\\downarrow", "v");
    m.insert("\\leftrightarrow", "<->");
    m.insert("\\Leftarrow", "<=");
    m.insert("\\Rightarrow", "=>");
    m.insert("\\implies", "=>");
    m.insert("\\Leftrightarrow", "<=>");
    m.insert("\\iff", "<=>");
    m.insert("\\mapsto", "|->");

    // Big operators - use ASCII for TI-Nspire compatibility
    m.insert("\\sum", "SUM");
    m.insert("\\prod", "PROD");
    m.insert("\\coprod", "COPROD");
    m.insert("\\int", "INT");
    m.insert("\\oint", "OINT");
    m.insert("\\iint", "IINT");
    m.insert("\\iiint", "IIINT");
    m.insert("\\bigcup", "UNION");
    m.insert("\\bigcap", "INTERSECT");
    m.insert("\\bigoplus", "OPLUS");
    m.insert("\\bigotimes", "OTIMES");

    // Misc symbols - use ASCII for problematic ones
    m.insert("\\infty", "inf");  // infinity
    m.insert("\\partial", "d");  // partial derivative (use 'd')
    m.insert("\\nabla", "nabla");
    m.insert("\\forall", "forall");
    m.insert("\\exists", "exists");
    m.insert("\\nexists", "!exists");
    m.insert("\\emptyset", "{}");
    m.insert("\\varnothing", "{}");
    m.insert("\\neg", "NOT");
    m.insert("\\lnot", "NOT");
    m.insert("\\land", "AND");
    m.insert("\\wedge", "AND");
    m.insert("\\lor", "OR");
    m.insert("\\vee", "OR");
    m.insert("\\cap", "n");  // intersection (simple)
    m.insert("\\cup", "U");  // union (simple)
    m.insert("\\setminus", "\\");
    m.insert("\\angle", "<");
    m.insert("\\triangle", "^");
    m.insert("\\square", "[]");
    m.insert("\\diamond", "<>");
    m.insert("\\clubsuit", "club");
    m.insert("\\diamondsuit", "diamond");
    m.insert("\\heartsuit", "heart");
    m.insert("\\spadesuit", "spade");
    m.insert("\\aleph", "aleph");
    m.insert("\\wp", "P");
    m.insert("\\Re", "Re");
    m.insert("\\Im", "Im");
    m.insert("\\hbar", "hbar");
    m.insert("\\ell", "l");
    m.insert("\\prime", "'");
    m.insert("\\degree", "deg");
    m.insert("\\deg", "deg");

    // Roots and fractions (simple representations)
    m.insert("\\sqrt", "√");
    m.insert("\\cbrt", "∛");
    m.insert("\\frac12", "½");
    m.insert("\\frac13", "⅓");
    m.insert("\\frac23", "⅔");
    m.insert("\\frac14", "¼");
    m.insert("\\frac34", "¾");
    m.insert("\\frac15", "⅕");
    m.insert("\\frac25", "⅖");
    m.insert("\\frac35", "⅗");
    m.insert("\\frac45", "⅘");
    m.insert("\\frac16", "⅙");
    m.insert("\\frac56", "⅚");
    m.insert("\\frac18", "⅛");
    m.insert("\\frac38", "⅜");
    m.insert("\\frac58", "⅝");
    m.insert("\\frac78", "⅞");

    // Special spacing and formatting
    m.insert("\\,", " ");      // thin space
    m.insert("\\;", " ");      // medium space
    m.insert("\\:", " ");      // medium space
    m.insert("\\!", "");       // negative thin space (remove)
    m.insert("\\quad", "  ");  // quad space
    m.insert("\\qquad", "    "); // double quad
    m.insert("\\ldots", "…");
    m.insert("\\cdots", "⋯");
    m.insert("\\vdots", "⋮");
    m.insert("\\ddots", "⋱");

    // Brackets
    m.insert("\\langle", "⟨");
    m.insert("\\rangle", "⟩");
    m.insert("\\lceil", "⌈");
    m.insert("\\rceil", "⌉");
    m.insert("\\lfloor", "⌊");
    m.insert("\\rfloor", "⌋");
    m.insert("\\lvert", "|");
    m.insert("\\rvert", "|");
    m.insert("\\|", "‖");
    m.insert("\\lVert", "‖");
    m.insert("\\rVert", "‖");

    m
});

/// Superscript digit mapping
static SUPERSCRIPTS: LazyLock<HashMap<char, char>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert('0', '⁰');
    m.insert('1', '¹');
    m.insert('2', '²');
    m.insert('3', '³');
    m.insert('4', '⁴');
    m.insert('5', '⁵');
    m.insert('6', '⁶');
    m.insert('7', '⁷');
    m.insert('8', '⁸');
    m.insert('9', '⁹');
    m.insert('+', '⁺');
    m.insert('-', '⁻');
    m.insert('=', '⁼');
    m.insert('(', '⁽');
    m.insert(')', '⁾');
    m.insert('n', 'ⁿ');
    m.insert('i', 'ⁱ');
    m.insert('x', 'ˣ');
    m.insert('y', 'ʸ');
    m
});

/// Subscript digit mapping
static SUBSCRIPTS: LazyLock<HashMap<char, char>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert('0', '₀');
    m.insert('1', '₁');
    m.insert('2', '₂');
    m.insert('3', '₃');
    m.insert('4', '₄');
    m.insert('5', '₅');
    m.insert('6', '₆');
    m.insert('7', '₇');
    m.insert('8', '₈');
    m.insert('9', '₉');
    m.insert('+', '₊');
    m.insert('-', '₋');
    m.insert('=', '₌');
    m.insert('(', '₍');
    m.insert(')', '₎');
    m.insert('a', 'ₐ');
    m.insert('e', 'ₑ');
    m.insert('h', 'ₕ');
    m.insert('i', 'ᵢ');
    m.insert('j', 'ⱼ');
    m.insert('k', 'ₖ');
    m.insert('l', 'ₗ');
    m.insert('m', 'ₘ');
    m.insert('n', 'ₙ');
    m.insert('o', 'ₒ');
    m.insert('p', 'ₚ');
    m.insert('r', 'ᵣ');
    m.insert('s', 'ₛ');
    m.insert('t', 'ₜ');
    m.insert('u', 'ᵤ');
    m.insert('v', 'ᵥ');
    m.insert('x', 'ₓ');
    m
});

/// Convert LaTeX-style math notation to Unicode
///
/// Supports:
/// - Greek letters: \alpha, \beta, \Gamma, etc.
/// - Operators: \times, \div, \pm, \leq, \geq, etc.
/// - Symbols: \infty, \sum, \int, \partial, etc.
/// - Superscripts: x^2 → x², x^{10} → x¹⁰
/// - Subscripts: x_1 → x₁, x_{10} → x₁₀
/// - Simple fractions: \frac12 → ½
pub fn latex_to_unicode(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '\\' {
            // Try to match a LaTeX command
            if let Some((replacement, consumed)) = try_match_command(&chars, i) {
                result.push_str(replacement);
                i += consumed;
                continue;
            }
        } else if chars[i] == '^' {
            // Superscript
            if let Some((superscript, consumed)) = convert_script(&chars, i + 1, &SUPERSCRIPTS) {
                result.push_str(&superscript);
                i += 1 + consumed;
                continue;
            }
        } else if chars[i] == '_' {
            // Subscript
            if let Some((subscript, consumed)) = convert_script(&chars, i + 1, &SUBSCRIPTS) {
                result.push_str(&subscript);
                i += 1 + consumed;
                continue;
            }
        }

        result.push(chars[i]);
        i += 1;
    }

    result
}

/// Try to match a LaTeX command starting at position `start`
fn try_match_command(chars: &[char], start: usize) -> Option<(&'static str, usize)> {
    // Build the command string character by character
    let mut cmd = String::with_capacity(16);
    let mut i = start;

    // Include the backslash
    cmd.push(chars[i]);
    i += 1;

    // Collect command name (letters only for standard commands)
    while i < chars.len() && chars[i].is_ascii_alphabetic() {
        cmd.push(chars[i]);
        i += 1;
    }

    // Check for special commands like \frac12
    if cmd == "\\frac" && i + 1 < chars.len() {
        let frac_cmd = format!("\\frac{}{}", chars[i], chars[i + 1]);
        if let Some(&replacement) = LATEX_MAP.get(frac_cmd.as_str()) {
            return Some((replacement, i + 2 - start));
        }
    }

    // Try to find in map
    if let Some(&replacement) = LATEX_MAP.get(cmd.as_str()) {
        return Some((replacement, i - start));
    }

    // Check for single-char special commands like \, \; \: \!
    if start + 1 < chars.len() {
        let special_cmd: String = chars[start..=start + 1].iter().collect();
        if let Some(&replacement) = LATEX_MAP.get(special_cmd.as_str()) {
            return Some((replacement, 2));
        }
    }

    None
}

/// Convert characters after ^ or _ to super/subscript
fn convert_script(chars: &[char], start: usize, map: &HashMap<char, char>) -> Option<(String, usize)> {
    if start >= chars.len() {
        return None;
    }

    let mut result = String::new();
    let mut all_converted = true;
    let mut original_text = String::new();

    let consumed = if chars[start] == '{' {
        // Braced group: ^{123} or _{abc}
        let mut i = start + 1;
        while i < chars.len() && chars[i] != '}' {
            original_text.push(chars[i]);
            if let Some(&converted) = map.get(&chars[i]) {
                result.push(converted);
            } else {
                // Can't convert this character - mark that not all converted
                all_converted = false;
                result.push(chars[i]);
            }
            i += 1;
        }
        if i < chars.len() && chars[i] == '}' {
            i - start + 1
        } else {
            return None; // Unclosed brace
        }
    } else {
        // Single character: ^2 or _1
        original_text.push(chars[start]);
        if let Some(&converted) = map.get(&chars[start]) {
            result.push(converted);
            1
        } else {
            // Can't convert this single character
            // Use parentheses fallback to avoid blank rectangles on TI-Nspire
            all_converted = false;
            result.push(chars[start]);
            1
        }
    };

    if result.is_empty() {
        None
    } else {
        // If not all characters could be converted in a braced group,
        // use regular text in parentheses instead for TI-Nspire compatibility
        // The ^ and _ characters don't render on TI-Nspire, so we must avoid them
        // Example: ^{abc} becomes (abc) instead of ^{abc} or trying unavailable superscripts
        if !all_converted {
            Some((format!("({})", original_text), consumed))
        } else {
            Some((result, consumed))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greek_letters() {
        assert_eq!(latex_to_unicode("\\alpha + \\beta = \\gamma"), "α + β = γ");
        // Uppercase Greek uses ASCII for TI-Nspire compatibility
        assert_eq!(latex_to_unicode("\\Sigma\\Omega"), "SigmaOmega");
        assert_eq!(latex_to_unicode("\\Delta x"), "Delta x");
    }

    #[test]
    fn test_operators() {
        assert_eq!(latex_to_unicode("a \\times b"), "a × b");
        assert_eq!(latex_to_unicode("x \\leq y \\leq z"), "x ≤ y ≤ z");
        assert_eq!(latex_to_unicode("a \\neq b"), "a ≠ b");
    }

    #[test]
    fn test_superscripts() {
        assert_eq!(latex_to_unicode("x^2"), "x²");
        assert_eq!(latex_to_unicode("x^{10}"), "x¹⁰");
        assert_eq!(latex_to_unicode("x^2 + y^2 = z^2"), "x² + y² = z²");
        // Note: LaTeX commands inside braces aren't parsed (would need recursion)
        // For complex expressions, convert LaTeX first: e^{i\pi} -> e^{i}π
        assert_eq!(latex_to_unicode("e^i\\pi"), "eⁱπ");
    }

    #[test]
    fn test_subscripts() {
        assert_eq!(latex_to_unicode("x_1"), "x₁");
        assert_eq!(latex_to_unicode("x_{12}"), "x₁₂");
        assert_eq!(latex_to_unicode("H_2O"), "H₂O");
        // 'n' has a subscript available
        assert_eq!(latex_to_unicode("a_n"), "aₙ");
    }

    #[test]
    fn test_symbols() {
        // Big operators and special symbols use ASCII for TI-Nspire
        assert_eq!(latex_to_unicode("\\infty"), "inf");
        assert_eq!(latex_to_unicode("\\sum_{i=0}^{n}"), "SUMᵢ₌₀ⁿ");
        assert_eq!(latex_to_unicode("\\int f(x) dx"), "INT f(x) dx");
        assert_eq!(latex_to_unicode("\\partial f"), "d f");
        assert_eq!(latex_to_unicode("\\nabla"), "nabla");
    }

    #[test]
    fn test_fractions() {
        assert_eq!(latex_to_unicode("\\frac12 + \\frac14 = \\frac34"), "½ + ¼ = ¾");
    }

    #[test]
    fn test_arrows() {
        // Arrows use ASCII for TI-Nspire compatibility
        assert_eq!(latex_to_unicode("a \\rightarrow b"), "a -> b");
        assert_eq!(latex_to_unicode("A \\Rightarrow B"), "A => B");
        assert_eq!(latex_to_unicode("P \\iff Q"), "P <=> Q");
        assert_eq!(latex_to_unicode("x \\to y"), "x -> y");  // Common in limits
    }

    #[test]
    fn test_mixed() {
        assert_eq!(
            latex_to_unicode("E = mc^2"),
            "E = mc²"
        );
        // \forall uses ASCII "forall" for TI-Nspire compatibility
        assert_eq!(
            latex_to_unicode("\\forall x \\in \\mathbb{R}: x^2 \\geq 0"),
            "forall x in \\mathbb{R}: x² ≥ 0"  // \in becomes "in", \mathbb not supported
        );
    }

    #[test]
    fn test_no_latex() {
        assert_eq!(latex_to_unicode("Hello World"), "Hello World");
        assert_eq!(latex_to_unicode("2 + 2 = 4"), "2 + 2 = 4");
    }

    #[test]
    fn test_partial_match() {
        // Unknown commands are kept as-is
        assert_eq!(latex_to_unicode("\\unknown"), "\\unknown");
    }
}
