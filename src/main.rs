use rand::Rng;
use std::cmp::Ordering;
use std::io;

fn main() {
    println!("Guess the number!");
    println!("Please input your guess.");

    let tup = (500, 6.4, 1);
    match tup {
        (x, _, _) => println!("first value is {}", x),
    }

    let secret_number = rand::thread_rng().gen_range(1, 101);
    loop {
        let mut guess = String::new();
        io::stdin()
            .read_line(&mut guess)
            .expect("Failed to read line");

        let guess: u32 = guess.trim().parse().expect("Please type a number!");
        match guess.cmp(&secret_number) {
            Ordering::Less => println!("Too small! {}", secret_number),
            Ordering::Greater => println!("Too big! {}", secret_number),
            Ordering::Equal => {
                println!("You win!");
                break;
            }
        }
    }
}
