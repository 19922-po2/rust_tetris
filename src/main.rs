use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor},
    terminal::{self, ClearType},
};
use rand::Rng;
use std::io::{Write, stdout};
use std::thread;
use std::time::{Duration, Instant};

const ROWS: usize = 20;
const COLS: usize = 10;

type Board = [[u8; COLS]; ROWS];

#[derive(Clone, Copy)]
enum BlockType {
    T,
    L,
    S,
    Z,
    I,
}

#[derive(Clone)]
struct Block {
    rotations: Vec<[(i32, i32); 4]>,
}

impl Block {
    fn get_cells(&self, rotation: usize) -> &[(i32, i32); 4] {
        &self.rotations[rotation % self.rotations.len()]
    }
}

struct Game {
    board: Board,
    falling_block: BlockType,
    rotation: usize,
    row: i32,
    col: i32,
    score: u32,
}

fn main() {
    let mut stdout = stdout();
    terminal::enable_raw_mode().unwrap();
    execute!(stdout, terminal::Clear(ClearType::All)).unwrap();

    let blocks = init_blocks();
    let mut game = Game {
        board: [[0; COLS]; ROWS],
        falling_block: random_block(),
        rotation: 0,
        row: 0,
        col: 3,
        score: 0,
    };

    // Input cooldown timers
    let mut last_up = Instant::now() - Duration::from_millis(200);
    let mut last_left = Instant::now() - Duration::from_millis(200);
    let mut last_right = Instant::now() - Duration::from_millis(200);
    let mut last_down = Instant::now() - Duration::from_millis(200);
    let key_cooldown = Duration::from_millis(150);

    // Falling timer
    let mut fall_timer = 0;
    let fall_interval = 400; // milliseconds

    loop {
        render(&game, &blocks);

        // Handle key input
        if event::poll(Duration::from_millis(20)).unwrap() {
            if let Event::Key(key_event) = event::read().unwrap() {
                match key_event.code {
                    KeyCode::Up => {
                        if last_up.elapsed() >= key_cooldown {
                            rotate_block(&mut game, &blocks);
                            last_up = Instant::now();
                        }
                    }
                    KeyCode::Left => {
                        if last_left.elapsed() >= key_cooldown {
                            try_move(&mut game, &blocks, -1, 0);
                            last_left = Instant::now();
                        }
                    }
                    KeyCode::Right => {
                        if last_right.elapsed() >= key_cooldown {
                            try_move(&mut game, &blocks, 1, 0);
                            last_right = Instant::now();
                        }
                    }
                    KeyCode::Down => {
                        if last_down.elapsed() >= key_cooldown {
                            try_move(&mut game, &blocks, 0, 1);
                            last_down = Instant::now();
                        }
                    }
                    KeyCode::Char('q') => break,
                    _ => {}
                }
            }
        }

        // Falling logic
        fall_timer += 20;
        if fall_timer >= fall_interval {
            fall_timer = 0;
            if !try_move(&mut game, &blocks, 0, 1) {
                settle_block(&mut game, &blocks);
                collapse_rows(&mut game);
                game.falling_block = random_block();
                game.rotation = 0;
                game.row = 0;
                game.col = 3;

                if !is_drawable(&game, &blocks, game.row, game.col, game.rotation) {
                    execute!(stdout, terminal::Clear(ClearType::All)).unwrap();
                    println!("Game Over! Final Score: {}", game.score);
                    break;
                }
            }
        }

        thread::sleep(Duration::from_millis(20));
    }

    terminal::disable_raw_mode().unwrap();
}

fn init_blocks() -> Vec<Block> {
    vec![
        // T Block
        Block {
            rotations: vec![
                [(0, 1), (1, 0), (1, 1), (1, 2)],
                [(0, 1), (1, 1), (1, 2), (2, 1)],
                [(1, 0), (1, 1), (1, 2), (2, 1)],
                [(0, 1), (1, 0), (1, 1), (2, 1)],
            ],
        },
        // L Block
        Block {
            rotations: vec![
                [(0, 0), (1, 0), (1, 1), (1, 2)],
                [(0, 1), (0, 2), (1, 1), (2, 1)],
                [(1, 0), (1, 1), (1, 2), (2, 2)],
                [(0, 1), (1, 1), (2, 0), (2, 1)],
            ],
        },
        // S Block
        Block {
            rotations: vec![
                [(0, 1), (0, 2), (1, 0), (1, 1)],
                [(0, 0), (1, 0), (1, 1), (2, 1)],
            ],
        },
        // Z Block
        Block {
            rotations: vec![
                [(0, 0), (0, 1), (1, 1), (1, 2)],
                [(0, 1), (1, 0), (1, 1), (2, 0)],
            ],
        },
        // I Block
        Block {
            rotations: vec![
                [(0, 0), (1, 0), (2, 0), (3, 0)],
                [(0, 0), (0, 1), (0, 2), (0, 3)],
            ],
        },
    ]
}

fn random_block() -> BlockType {
    match rand::thread_rng().gen_range(0..5) {
        0 => BlockType::T,
        1 => BlockType::L,
        2 => BlockType::S,
        3 => BlockType::Z,
        _ => BlockType::I,
    }
}

fn block_index(block: BlockType) -> usize {
    match block {
        BlockType::T => 0,
        BlockType::L => 1,
        BlockType::S => 2,
        BlockType::Z => 3,
        BlockType::I => 4,
    }
}

fn render(game: &Game, blocks: &[Block]) {
    let mut stdout = stdout();
    execute!(stdout, terminal::Clear(ClearType::All)).unwrap();

    // Draw board
    for r in 0..ROWS {
        for c in 0..COLS {
            if game.board[r][c] > 0 {
                execute!(
                    stdout,
                    cursor::MoveTo(c as u16 * 2, r as u16),
                    SetBackgroundColor(Color::Cyan),
                    Print("  "),
                    ResetColor
                )
                .unwrap();
            }
        }
    }

    // Draw falling block
    let idx = block_index(game.falling_block);
    for (dr, dc) in blocks[idx].get_cells(game.rotation) {
        let r = game.row + dr;
        let c = game.col + dc;
        if r >= 0 && r < ROWS as i32 && c >= 0 && c < COLS as i32 {
            execute!(
                stdout,
                cursor::MoveTo(c as u16 * 2, r as u16),
                SetBackgroundColor(Color::Blue),
                Print("  "),
                ResetColor
            )
            .unwrap();
        }
    }

    // Draw score
    execute!(
        stdout,
        cursor::MoveTo(0, ROWS as u16 + 1),
        Print(format!("Score: {}", game.score))
    )
    .unwrap();

    stdout.flush().unwrap();
}

fn is_drawable(game: &Game, blocks: &[Block], row: i32, col: i32, rotation: usize) -> bool {
    let idx = block_index(game.falling_block);
    for (dr, dc) in blocks[idx].get_cells(rotation) {
        let r = row + dr;
        let c = col + dc;
        if r < 0 || r >= ROWS as i32 || c < 0 || c >= COLS as i32 {
            return false;
        }
        if game.board[r as usize][c as usize] > 0 {
            return false;
        }
    }
    true
}

fn try_move(game: &mut Game, blocks: &[Block], dc: i32, dr: i32) -> bool {
    let new_row = game.row + dr;
    let new_col = game.col + dc;
    if is_drawable(game, blocks, new_row, new_col, game.rotation) {
        game.row = new_row;
        game.col = new_col;
        true
    } else {
        false
    }
}

fn rotate_block(game: &mut Game, blocks: &[Block]) {
    let next_rotation =
        (game.rotation + 1) % blocks[block_index(game.falling_block)].rotations.len();

    // Try rotation in place
    if is_drawable(game, blocks, game.row, game.col, next_rotation) {
        game.rotation = next_rotation;
        return;
    }
    // Wall kick left
    if is_drawable(game, blocks, game.row, game.col - 1, next_rotation) {
        game.col -= 1;
        game.rotation = next_rotation;
        return;
    }
    // Wall kick right
    if is_drawable(game, blocks, game.row, game.col + 1, next_rotation) {
        game.col += 1;
        game.rotation = next_rotation;
        return;
    }
    // Cannot rotate
}

fn settle_block(game: &mut Game, blocks: &[Block]) {
    let idx = block_index(game.falling_block);
    for (dr, dc) in blocks[idx].get_cells(game.rotation) {
        let r = game.row + dr;
        let c = game.col + dc;
        if r >= 0 && r < ROWS as i32 && c >= 0 && c < COLS as i32 {
            game.board[r as usize][c as usize] = 1;
        }
    }
}

fn collapse_rows(game: &mut Game) {
    for r in (0..ROWS).rev() {
        if game.board[r].iter().all(|&x| x > 0) {
            game.score += 10;
            for rr in (1..=r).rev() {
                game.board[rr] = game.board[rr - 1];
            }
            game.board[0] = [0; COLS];
        }
    }
}
