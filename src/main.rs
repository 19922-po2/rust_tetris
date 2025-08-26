//use std::io::Write;
use std::thread;
use std::time::Duration;

const BOARD_WIDTH: usize = 10;
const BOARD_HEIGHT: usize = 25; // board is 20 height visible and 5 hidden

fn clear_terminal() {
    print!("\x1B[2J\x1B[H"); // Clear the screen and move the cursor to the top-left corner
    //std::io::stdout().flush().unwrap(); // Ensure the output is flushed immediately
}

fn draw_empty_or_block(x: usize, y: usize, game_board: &Vec<Vec<u32>>) {
    if game_board[y][x] == 0 {
        print!("  ");
    } else {
        print!("\u{2B1C}");
    }
}

fn draw_game(game_board: &Vec<Vec<u32>>) {
    println!();
    for y in 5..game_board.len() {
        for x in 0..game_board[y].len() + 1 {
            if x == 0 || x == 11 || y == BOARD_HEIGHT {
                print!("\u{2B1B}") // 2B1C white square, 2B1B black square
            } else {
                //print!("  ");
                draw_empty_or_block(x, y, game_board);
            }
        }
        println!();
    }
}

fn move_block(game_board: &mut Vec<Vec<u32>>, current_block: &mut Vec<(usize, usize)>) {
    for (x, y) in current_block.clone() {
        game_board[y][x] = 0;
        game_board[y + 1][x] = 1;
        current_block.push((x, y + 1));
        current_block.remove(0);
    }
}

fn main() {
    clear_terminal();
    println!("Tetris Game");

    let mut game_board: Vec<Vec<u32>> = vec![vec![0; BOARD_WIDTH + 1]; BOARD_HEIGHT + 1];
    let mut current_block: Vec<(usize, usize)> = vec![(4, 4), (4, 5)];

    //draw_game(&game_board);
    loop {
        println!("{:?}", &current_block);
        // Game loop
        clear_terminal();
        move_block(&mut game_board, &mut current_block);
        draw_game(&game_board);

        thread::sleep(Duration::from_secs(1));
    }
}
