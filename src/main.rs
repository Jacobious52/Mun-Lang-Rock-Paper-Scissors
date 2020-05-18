use std::{cell::RefCell, cmp::Ordering, env, rc::Rc};

use mun_runtime::{invoke_fn, RetryResultExt, Runtime, RuntimeBuilder};
use rand::Rng;

const ROCK: u64 = 0;
const PAPER: u64 = 1;
const SCISSORS: u64 = 2;

const COUNTER_INIT: u64 = 10_000;

extern "C" fn random_move() -> u64 {
    let mut rng = rand::thread_rng();
    rng.gen::<u64>() % 3
}

extern "C" fn rock() -> u64 {
    ROCK
}

extern "C" fn paper() -> u64 {
    PAPER
}

extern "C" fn scissors() -> u64 {
    SCISSORS
}

fn load_program(lib_path: &str) -> Rc<RefCell<Runtime>> {
    RuntimeBuilder::new(lib_path)
        .insert_fn("random_move", random_move as extern "C" fn() -> u64)
        .insert_fn("rock", rock as extern "C" fn() -> u64)
        .insert_fn("paper", paper as extern "C" fn() -> u64)
        .insert_fn("scissors", scissors as extern "C" fn() -> u64)
        .spawn()
        .expect("Failed to spawn Runtime")
}

#[derive(Debug, PartialEq)]
struct Move(u64);

impl PartialOrd for Move {
    fn partial_cmp(&self, other: &Move) -> Option<Ordering> {
        assert!(self.0 > 0);
        assert!(self.0 <= 3);
        assert!(other.0 > 0);
        assert!(other.0 <= 3);

        if self.0 == other.0 {
            return Some(Ordering::Equal);
        }

        let win_table = [SCISSORS, ROCK, PAPER];

        if win_table[self.0 as usize] == other.0 {
            return Some(Ordering::Greater);
        }

        Some(Ordering::Less)
    }
}

fn print_scale(a: u64, b: u64, width: u64) {
    let percent_a =  if a == 0 && b == 0 {
        0.5
    } else {
        a as f64 / (a as f64 + b as f64)
    };

    let percent_a_to_w = (percent_a * width as f64) as u64;

    assert!(percent_a_to_w <= width);

    print!("P1");
    for _ in 0..percent_a_to_w {
        print!(">");
    }

    print!("|{:.4}|", percent_a);

    for _ in percent_a_to_w..width {
        print!("<");
    }
    print!("P2");
    print!("\r");
}

fn main() {
    let lib_path_player_1 = env::args()
        .nth(1)
        .expect("expected path to a mun library player 1");
    let lib_path_player_2 = env::args()
        .nth(2)
        .expect("expected path to a mun library player 2");

    let player_1 = load_program(&lib_path_player_1);
    let player_2 = load_program(&lib_path_player_2);

    // reduce the bar sensitivity at the start
    let mut player_1_counter: u64 = COUNTER_INIT;
    let mut player_2_counter: u64 = COUNTER_INIT;

    let mut last_player_1_move = 0;
    let mut last_player_2_move = 0;

    loop {
        let player_1_move: u64 = invoke_fn!(player_1, "next_move", last_player_2_move).wait();
        let player_2_move: u64 = invoke_fn!(player_2, "next_move", last_player_1_move).wait();

        last_player_1_move = player_1_move;
        last_player_2_move = player_2_move;

        let result = last_player_1_move.partial_cmp(&last_player_2_move).unwrap();
        match result {
            Ordering::Greater => player_1_counter += 1,
            Ordering::Less => player_2_counter += 1,
            Ordering::Equal => {}
        }
        let _winner = match result {
            Ordering::Greater => "player 1 wins",
            Ordering::Less => "player 2 wins",
            Ordering::Equal => "draw",
        };

        // print!("{} - {} : p1 {}, p2 {} : {}\r",  player_1_counter, player_2_counter, player_1_move, player_2_move, winner);
        print_scale(player_1_counter, player_2_counter, 100);


        let updated_1 = player_1.borrow_mut().update();
        let updated_2 = player_2.borrow_mut().update();

        if updated_1 || updated_2 {
            player_1_counter = COUNTER_INIT;
            player_2_counter = COUNTER_INIT;
        }
    }
}
