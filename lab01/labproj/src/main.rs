use rand::RngExt;
use std::{
    fs::File,
    io::{self, Write},
};

const TAB_SIZE: usize = 10;

fn generate_powers_table(x: u64) -> [u64; TAB_SIZE] {
    let mut num = 1;
    let mut tab: [u64; TAB_SIZE] = [0; TAB_SIZE];

    // xD
    let iter = 0..TAB_SIZE;
    for i in iter {
        tab[i] = num;
        num *= x;
    }
    tab

    //     znajdź jedną różnicę
    //     for i in 0..TAB_SIZE {
    //         tab[i] = num;
    //         num *= x;
    //     }
    //     tab
}

fn test_collatz_on_table(tab: &[u64]) -> [bool; TAB_SIZE] {
    let mut answ_tab: [bool; TAB_SIZE] = [false; TAB_SIZE];

    for i in 0..TAB_SIZE {
        let mut c = tab[i];

        for _j in 0..100 {
            if c == 1 {
                answ_tab[i] = true;
                break;
            } else if c.is_multiple_of(2) {
                c /= 2;
            } else {
                c = 3 * c + 1;
            }
        }
    }

    answ_tab
}

fn generate_mini_passing_cookie() -> (f32, String) {
    let mut cookie = (0.0, String::from("sopów"));

    cookie.0 = 'main: loop {
        let rand_range = 'range: loop {
            let zakres: (f32, f32) = (
                rand::rng().random_range(0.0..=1.0),
                rand::rng().random_range(0.0..=1.0),
            );
            if zakres.0 > zakres.1 {
                continue;
            } else if zakres.0 == zakres.1 {
                break 'main zakres.0;
            } else {
                break 'range zakres;
            }
        };

        'decyzja: loop {
            println!(
                "Wylosowany zakres losowania {}–{}.",
                rand_range.0, rand_range.1
            );
            println!("Losować ponownie? (y/n)");

            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();

            match input.trim() {
                "y" => break 'decyzja,
                "n" => {
                    break 'main rand::rng().random_range(rand_range.0..=rand_range.1);
                }
                _ => println!("usage!"),
            }
        }
    };

    cookie
}
fn interpret_fortune_cookie(cookie: &(f32, String)) {
    println!(
        "Szanse na zdanie przez ciebie {} wynoszą {}%!",
        cookie.1,
        cookie.0 * 100.0
    );
    if cookie.0 > 90.0 / 100.0 {
        println!("brawo nerdzie!");
    }
}

fn main() {
    let loop_result = loop {
        println!("Podaj liczbę");

        let mut inputline = String::new();
        io::stdin()
            .read_line(&mut inputline)
            .expect("failed getting line");

        let mut start_number: u64 = match inputline.trim().parse() {
            Ok(num) => num,
            Err(_) => break true,
        };

        if start_number == 0 {
            break false;
        }
        println!("liczba = {start_number}");
        let add_value = rand::rng().random_range(0..=5);
        start_number += add_value;
        println!("liczba = {start_number}");

        let powers_table = generate_powers_table(start_number);
        let collatz_test_val_table = test_collatz_on_table(&powers_table);

        let mut writing_file = File::create("xyz.txt").expect("failed creating a file");
        let mut file_string = String::new();
        for i in 0..TAB_SIZE {
            file_string.push_str(&format!(
                "{}\t\t{}\n",
                powers_table[i], collatz_test_val_table[i]
            ));
        }
        writing_file
            .write_all(file_string.as_bytes())
            .expect("error writing to file");
    };

    match loop_result {
        true => {
            println!("programn ended becouse of error");
        }
        false => {
            println!("Program ended by user");
        }
    }
    let cookie = generate_mini_passing_cookie();
    interpret_fortune_cookie(&cookie);
}
