use num_bigint::BigInt;
use num_traits::pow::Pow;
use rustyline::Editor;

/// Token types that can be parsed from input expressions
enum Token {
    Number(f64),
    Operator(char),
}

/// Main application entry point
fn main() {
    let mut rl = Editor::<()>::new().unwrap();
    println!("rcalc - Rust Calculator");
    println!("Enter expressions like '123+456' or '123*1e6'");
    println!("Press Ctrl+C to exit");

    loop {
        // Get user input with line editing support
        let readline = match rl.readline("> ") {
            Ok(line) => line,
            Err(_) => break,
        };

        // Add input to history
        rl.add_history_entry(&readline);

        // Remove all whitespace
        let input = readline.replace(" ", "");

        // Skip if empty
        if input.is_empty() {
            continue;
        }

        // Parse and evaluate the expression
        match evaluate_expression(&input) {
            Ok(result) => println!("{}", format_full_decimal(result)),
            Err(err) => println!("Error: {}", err),
        }
    }
}

/// Parses and evaluates a mathematical expression
///
/// Supports basic operators (+, -, *, /) and scientific notation (1e6)
/// Evaluates expressions from left to right without operator precedence
fn evaluate_expression(expr: &str) -> Result<f64, String> {
    let tokens = tokenize(expr)?;
    calculate(tokens)
}

/// Breaks an expression string into tokens (numbers and operators)
fn tokenize(expr: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut current_num = String::new();
    let mut i = 0;
    let chars: Vec<char> = expr.chars().collect();

    while i < chars.len() {
        let c = chars[i];

        if c.is_digit(10) || c == '.' || (c == 'e' || c == 'E') {
            // Handle digits, decimal points, and scientific notation
            current_num.push(c);

            // For scientific notation, also include the following sign and digits
            if (c == 'e' || c == 'E') && i + 1 < chars.len() {
                if chars[i + 1] == '+' || chars[i + 1] == '-' {
                    current_num.push(chars[i + 1]);
                    i += 1;
                }
            }
        } else if c == '+' || c == '-' || c == '*' || c == '/' {
            // When we encounter an operator, finalize the current number token
            if !current_num.is_empty() {
                let num = current_num.parse::<f64>()
                    .map_err(|_| format!("Invalid number: {}", current_num))?;
                tokens.push(Token::Number(num));
                current_num.clear();
            }
            tokens.push(Token::Operator(c));
        } else {
            return Err(format!("Invalid character: {}", c));
        }
        i += 1;
    }

    // Process the final number if there is one
    if !current_num.is_empty() {
        let num = current_num.parse::<f64>()
            .map_err(|_| format!("Invalid number: {}", current_num))?;
        tokens.push(Token::Number(num));
    }

    Ok(tokens)
}

/// Performs calculation on the provided tokens
fn calculate(tokens: Vec<Token>) -> Result<f64, String> {
    let mut result = 0.0;
    let mut current_op = '+'; // Start with addition (0 + first_number)

    for token in tokens {
        match token {
            Token::Number(num) => {
                result = apply_operation(result, num, current_op)?;
            },
            Token::Operator(op) => {
                current_op = op;
            }
        }
    }

    Ok(result)
}

/// Applies a single operation between two numbers
fn apply_operation(left: f64, right: f64, op: char) -> Result<f64, String> {
    match op {
        '+' => Ok(left + right),
        '-' => Ok(left - right),
        '*' => Ok(left * right),
        '/' => {
            if right == 0.0 {
                Err("Division by zero!".to_string())
            } else {
                Ok(left / right)
            }
        }
        _ => Err(format!("Invalid operator: {}", op)),
    }
}

/// Formats a number to display full decimal representation without scientific notation
///
/// Uses BigInt for handling very large numbers and preserves exact decimal representation
fn format_full_decimal(num: f64) -> String {
    if num.is_nan() {
        return "NaN".to_string();
    }
    if num.is_infinite() {
        return if num.is_sign_positive() {
            "Infinity".to_string()
        } else {
            "-Infinity".to_string()
        };
    }

    // For small numbers or numbers without scientific notation needed
    if num.abs() < 1e16 && num.abs() >= 1e-6 && !num.to_string().contains('e') {
        return format_regular_number(num);
    }

    // For numbers that need scientific notation handling
    let s = format!("{:e}", num);
    let parts: Vec<&str> = s.split('e').collect();
    let mantissa = parts[0].parse::<f64>().unwrap();
    let exp = parts[1].parse::<i32>().unwrap();

    // Use BigInt for precise decimal representation
    let mut result = if mantissa >= 0.0 {
        format_with_bigint(mantissa, exp)
    } else {
        let positive_result = format_with_bigint(-mantissa, exp);
        format!("-{}", positive_result)
    };

    // Trim trailing zeros for decimal numbers
    trim_trailing_zeros(&mut result);
    result
}

/// Formats a number that doesn't require scientific notation handling
fn format_regular_number(num: f64) -> String {
    let mut s = format!("{}", num);
    trim_trailing_zeros(&mut s);
    s
}

/// Removes trailing zeros and decimal point if needed
fn trim_trailing_zeros(s: &mut String) {
    if s.contains('.') {
        while s.ends_with('0') {
            s.pop();
        }
        if s.ends_with('.') {
            s.pop();
        }
    }
}

/// Formats a number using BigInt for precise decimal representation
///
/// Handles both very large and very small numbers with exact precision
fn format_with_bigint(mantissa: f64, exp: i32) -> String {
    // Format the mantissa and extract parts
    let mantissa_formatted = format!("{:.15}", mantissa);
    let mantissa_str = mantissa_formatted.trim_end_matches('0').trim_end_matches('.');
    let decimal_pos = mantissa_str.find('.');

    let (int_part, frac_part) = match decimal_pos {
        Some(pos) => {
            let int = &mantissa_str[..pos];
            let frac = &mantissa_str[pos + 1..];
            (int, frac)
        }
        None => (mantissa_str, ""),
    };

    let mantissa_as_int = format!("{}{}", int_part, frac_part);
    let digits_moved = frac_part.len() as i32;

    // Adjust exponent to account for the decimal point removal
    let adjusted_exp = exp - digits_moved;

    if adjusted_exp >= 0 {
        // For positive exponents (larger numbers)
        format_large_number(&mantissa_as_int, adjusted_exp)
    } else {
        // For negative exponents (smaller numbers)
        format_small_number(&mantissa_as_int, adjusted_exp)
    }
}

/// Formats a very large number (with positive exponent)
fn format_large_number(mantissa_str: &str, exp: i32) -> String {
    let base = BigInt::parse_bytes(mantissa_str.as_bytes(), 10).unwrap();
    let multiplier = BigInt::from(10).pow(exp as u32);
    let result = base * multiplier;
    format!("{}", result)
}

/// Formats a very small number (with negative exponent)
fn format_small_number(mantissa_str: &str, exp: i32) -> String {
    let neg_exp = -exp as usize;
    let base = BigInt::parse_bytes(mantissa_str.as_bytes(), 10).unwrap();

    let result = format!("{}", base);
    if neg_exp >= result.len() {
        // Number is smaller than 1
        let zeros_needed = neg_exp - result.len();
        format!("0.{}{}", "0".repeat(zeros_needed), result)
    } else {
        // Place decimal point
        let decimal_pos = result.len() - neg_exp;
        format!("{}.{}", &result[..decimal_pos], &result[decimal_pos..])
    }
}
