use std::io;

fn main() {
    loop {
        // Get user input
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read input");

        let input = input.trim(); // Remove trailing newline

        // Split input into parts
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.len() != 3 {
            println!("Invalid format! Use: number operator number");
            continue;
        }

        // Parse numbers and operator
        let num1: f64 = match parts[0].parse() {
            Ok(n) => n,
            Err(_) => {
                println!("Invalid first number!");
                continue;
            }
        };

        let operator = parts[1];
        let num2: f64 = match parts[2].parse() {
            Ok(n) => n,
            Err(_) => {
                println!("Invalid second number!");
                continue;
            }
        };

        // Perform calculation
        let result = match operator {
            "+" => num1 + num2,
            "-" => num1 - num2,
            "*" => num1 * num2,
            "/" => {
                if num2 == 0.0 {
                    println!("Error: Division by zero!");
                    continue;
                } else {
                    num1 / num2
                }
            }
            _ => {
                println!("Invalid operator! Use +, -, *, or /");
                continue;
            }
        };

        // Display result
        println!("{}", result);
    }
}
