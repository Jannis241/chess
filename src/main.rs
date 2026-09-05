//! Terminal UI entry point and runtime orchestration.
//!
//! This binary wires together:
//! - keyboard-driven board interaction,
//! - game-state transitions from `game`,
//! - built-in engine turns from `engine`,
//! - optional external Stockfish turns via UCI.

mod engine;
mod game;

use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::time::Duration;

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{self, Clear, ClearType},
};
use engine::{Engine, EngineConfig};
use game::{Chess, Color, GameStatus, Move, PieceKind};

fn main() {
    if let Err(err) = run() {
        eprintln!("Fatal terminal error: {err}");
    }
}

/// Runs the interactive game loop until quit.
fn run() -> io::Result<()> {
    let mode = prompt_game_mode()?;
    let mut game = Chess::new();
    let engine = mode.engine_config().map(Engine::with_config);
    let mut stockfish = match mode.stockfish_config() {
        Some(config) => Some(StockfishClient::new(config)?),
        None => None,
    };
    let mut uci_history = String::new();
    let eval_engine = Engine::new(1);
    let mut stdout = io::stdout();
    let mut ui = TerminalUi::start()?;
    let mut cursor_sq = square_to_idx(4, 1);
    let mut selected_sq = None;
    let mut game_status = game.status();
    let mut eval_cp = eval_engine.evaluate_position(&game);
    let mut captures = captured_pieces_display(&game);
    let mut material_balance = material_balance_pawns(&game);
    let mut checked_kings = game.checked_king_squares();
    let mut status_line = match &mode {
        GameMode::HumanVsHuman => {
            "Arrow keys: move cursor | Enter: select/move | Esc: cancel selection | q: quit"
                .to_string()
        }
        GameMode::HumanVsEngine { engine_config, .. } => format!(
            "Arrow keys: move cursor | Enter: select/move | Esc: cancel selection | q: quit | You play White | {}",
            engine_config_summary(*engine_config)
        ),
        GameMode::HumanVsStockfish {
            stockfish_config, ..
        } => format!(
            "Arrow keys: move cursor | Enter: select/move | Esc: cancel selection | q: quit | You play White | Stockfish {} | {} ms",
            stockfish_config.path, stockfish_config.move_time_ms
        ),
    };

    loop {
        if game_status == GameStatus::Ongoing
            && let Some(bot_color) = mode.engine_color()
            && game.current_turn() == bot_color
        {
            selected_sq = None;
            status_line = match &mode {
                GameMode::HumanVsEngine { .. } => "Engine is thinking...".to_string(),
                GameMode::HumanVsStockfish { .. } => "Stockfish is thinking...".to_string(),
                GameMode::HumanVsHuman => String::new(),
            };
            draw(
                &mut stdout,
                &game,
                &mode,
                cursor_sq,
                selected_sq,
                &game_status,
                eval_cp,
                &captures,
                material_balance,
                &checked_kings,
                &status_line,
            )?;
            match &mode {
                GameMode::HumanVsEngine { .. } => {
                    if let Some(engine) = engine.as_ref() {
                        if let Some(result) = engine.best_move_timed(&game) {
                            let from = idx_to_square(result.mv.from);
                            let to = idx_to_square(result.mv.to);
                            match game.try_make_move(result.mv) {
                                Ok(()) => {
                                    append_uci_move(&mut uci_history, result.mv);
                                    refresh_cached_state(
                                        &game,
                                        &eval_engine,
                                        &mut game_status,
                                        &mut eval_cp,
                                        &mut captures,
                                        &mut material_balance,
                                        &mut checked_kings,
                                    );
                                    status_line = format!(
                                        "Engine played {from} -> {to} | eval {:+.2} | d{} | {} ms",
                                        result.eval as f32 / 100.0,
                                        result.depth_reached,
                                        result.elapsed.as_millis()
                                    );
                                }
                                Err(err) => {
                                    status_line = format!("Engine move failed: {err}");
                                }
                            }
                        } else {
                            status_line = "Engine has no legal move.".to_string();
                        }
                    }
                }
                GameMode::HumanVsStockfish { .. } => {
                    if let Some(client) = stockfish.as_mut() {
                        match client.best_move(&uci_history) {
                            Ok(Some(mv)) => {
                                let from = idx_to_square(mv.from);
                                let to = idx_to_square(mv.to);
                                match game.try_make_move(mv) {
                                    Ok(()) => {
                                        append_uci_move(&mut uci_history, mv);
                                        refresh_cached_state(
                                            &game,
                                            &eval_engine,
                                            &mut game_status,
                                            &mut eval_cp,
                                            &mut captures,
                                            &mut material_balance,
                                            &mut checked_kings,
                                        );
                                        status_line = format!("Stockfish played {from} -> {to}");
                                    }
                                    Err(err) => {
                                        status_line = format!("Stockfish move failed: {err}");
                                    }
                                }
                            }
                            Ok(None) => {
                                status_line = "Stockfish has no legal move.".to_string();
                            }
                            Err(err) => {
                                status_line = format!("Stockfish error: {err}");
                            }
                        }
                    }
                }
                GameMode::HumanVsHuman => {}
            }
            continue;
        }

        draw(
            &mut stdout,
            &game,
            &mode,
            cursor_sq,
            selected_sq,
            &game_status,
            eval_cp,
            &captures,
            material_balance,
            &checked_kings,
            &status_line,
        )?;

        let key = read_key()?;

        if game_status != GameStatus::Ongoing {
            match key {
                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc | KeyCode::Enter => break,
                _ => continue,
            }
        }

        match key {
            KeyCode::Left => cursor_sq = step_file(cursor_sq, -1),
            KeyCode::Right => cursor_sq = step_file(cursor_sq, 1),
            KeyCode::Up => cursor_sq = step_rank(cursor_sq, 1),
            KeyCode::Down => cursor_sq = step_rank(cursor_sq, -1),
            KeyCode::Esc => {
                selected_sq = None;
                status_line = "Selection cleared.".to_string();
            }
            KeyCode::Char('q') | KeyCode::Char('Q') => break,
            KeyCode::Enter => {
                if let Some(from) = selected_sq {
                    if cursor_sq == from {
                        selected_sq = None;
                        status_line = "Selection cleared.".to_string();
                    } else if let Some(piece) = game.piece_at(cursor_sq)
                        && piece.color == game.current_turn()
                    {
                        selected_sq = Some(cursor_sq);
                        status_line = format!("Selected {}", idx_to_square(cursor_sq));
                    } else {
                        let to = cursor_sq;
                        let wanted = Move {
                            from,
                            to,
                            promotion: None,
                        };
                        match game.try_make_move(wanted) {
                            Ok(()) => {
                                append_uci_move(&mut uci_history, wanted);
                                refresh_cached_state(
                                    &game,
                                    &eval_engine,
                                    &mut game_status,
                                    &mut eval_cp,
                                    &mut captures,
                                    &mut material_balance,
                                    &mut checked_kings,
                                );
                                status_line = format!(
                                    "Played {} -> {}",
                                    idx_to_square(from),
                                    idx_to_square(to)
                                );
                                selected_sq = None;
                            }
                            Err(err) => {
                                status_line = format!("Invalid move: {err}");
                            }
                        }
                    }
                } else {
                    match game.piece_at(cursor_sq) {
                        Some(piece) if piece.color == game.current_turn() => {
                            selected_sq = Some(cursor_sq);
                            status_line = format!("Selected {}", idx_to_square(cursor_sq));
                        }
                        Some(_) => {
                            status_line =
                                "That piece belongs to the other side. Select one of your own."
                                    .to_string();
                        }
                        None => {
                            status_line = "No piece on that square to select.".to_string();
                        }
                    }
                }
            }
            _ => {}
        }
    }

    ui.finish();
    println!("Exiting.");
    Ok(())
}

/// Recomputes cached position analytics after a successful move.
fn refresh_cached_state(
    game: &Chess,
    eval_engine: &Engine,
    game_status: &mut GameStatus,
    eval_cp: &mut i32,
    captures: &mut CapturedDisplay,
    material_balance: &mut i32,
    checked_kings: &mut Vec<usize>,
) {
    *game_status = game.status();
    *eval_cp = eval_engine.evaluate_position(game);
    *captures = captured_pieces_display(game);
    *material_balance = material_balance_pawns(game);
    checked_kings.clear();
    checked_kings.extend(game.checked_king_squares());
}

fn draw(
    stdout: &mut io::Stdout,
    game: &Chess,
    mode: &GameMode,
    cursor_sq: usize,
    selected_sq: Option<usize>,
    game_status: &GameStatus,
    eval_cp: i32,
    captures: &CapturedDisplay,
    material_balance: i32,
    checked_kings: &[usize],
    status_line: &str,
) -> io::Result<()> {
    execute!(stdout, cursor::MoveTo(0, 0), Clear(ClearType::All))?;
    let legal_targets = selected_sq.map_or_else(Vec::new, |from| game.legal_targets_from(from));
    let mut frame = String::new();
    frame.push_str("Terminal Chess\r\n");
    let board = game.render_with_selection(cursor_sq, selected_sq, &legal_targets, checked_kings);
    for line in board.lines() {
        frame.push_str(line);
        frame.push_str("\r\n");
    }
    frame.push_str(&format!("{}\r\n", position_header(game, mode)));
    frame.push_str(&format!("{}\r\n", eval_line(eval_cp)));
    frame.push_str(&format!(
        "Captured by White: {}\r\n",
        captures.white_captured_display
    ));
    frame.push_str(&format!(
        "Captured by Black: {}\r\n",
        captures.black_captured_display
    ));
    frame.push_str(&format!(
        "Material: {}\r\n",
        material_balance_line(material_balance)
    ));
    frame.push_str(
        "Highlights: cursor/selected=green | legal targets=olive | last move=gray | check=red\r\n",
    );

    match game_status {
        GameStatus::Ongoing => {}
        status => {
            frame.push_str(&format!("{}\r\n", game_result_line(status)));
            frame.push_str("Press Enter, Esc, or q to quit.\r\n");
        }
    }

    frame.push_str(&format!("Cursor: {}\r\n", idx_to_square(cursor_sq)));
    if let Some(from) = selected_sq {
        frame.push_str(&format!("Selected: {}\r\n", idx_to_square(from)));
    } else {
        frame.push_str("Selected: (none)\r\n");
    }
    frame.push_str(&format!("{status_line}\r\n"));
    stdout.write_all(frame.as_bytes())?;
    stdout.flush()?;
    Ok(())
}

/// Returns the current turn/mode header line.
fn position_header(game: &Chess, mode: &GameMode) -> String {
    let side = match game.current_turn() {
        Color::White => "White",
        Color::Black => "Black",
    };
    format!("Turn: {side} | Mode: {}", mode.label())
}

/// Converts game status into user-facing result text.
fn game_result_line(status: &GameStatus) -> String {
    match status {
        GameStatus::Checkmate { winner } => format!("Checkmate. {winner} wins."),
        GameStatus::Stalemate => "Draw by stalemate.".to_string(),
        GameStatus::DrawFiftyMove => "Draw by fifty-move rule.".to_string(),
        GameStatus::DrawThreefoldRepetition => "Draw by threefold repetition.".to_string(),
        GameStatus::DrawInsufficientMaterial => "Draw by insufficient material.".to_string(),
        GameStatus::Ongoing => String::new(),
    }
}

/// Blocks until a non-key-release key event is read.
fn read_key() -> io::Result<KeyCode> {
    loop {
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind == KeyEventKind::Release {
            continue;
        }
        return Ok(key.code);
    }
}

/// Converts board index to coordinate notation (`e4`).
fn idx_to_square(idx: usize) -> String {
    let file = (idx % 8) as u8;
    let rank = (idx / 8) as u8;
    let file_ch = (b'a' + file) as char;
    let rank_ch = (b'1' + rank) as char;
    format!("{file_ch}{rank_ch}")
}

/// Converts file/rank components to board index.
fn square_to_idx(file: usize, rank: usize) -> usize {
    rank * 8 + file
}

/// Converts board index to `(file, rank)` tuple.
fn idx_to_file_rank(idx: usize) -> (usize, usize) {
    (idx % 8, idx / 8)
}

/// Moves selection cursor horizontally with bounds clamping.
fn step_file(idx: usize, delta: i32) -> usize {
    let (file, rank) = idx_to_file_rank(idx);
    let new_file = (file as i32 + delta).clamp(0, 7) as usize;
    square_to_idx(new_file, rank)
}

/// Moves selection cursor vertically with bounds clamping.
fn step_rank(idx: usize, delta: i32) -> usize {
    let (file, rank) = idx_to_file_rank(idx);
    let new_rank = (rank as i32 + delta).clamp(0, 7) as usize;
    square_to_idx(file, new_rank)
}

struct TerminalUi;

impl TerminalUi {
    /// Enables raw terminal mode for real-time key handling.
    fn start() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        Ok(Self)
    }

    /// Restores terminal settings.
    fn finish(&mut self) {
        let _ = terminal::disable_raw_mode();
    }
}

impl Drop for TerminalUi {
    fn drop(&mut self) {
        self.finish();
    }
}

#[derive(Clone)]
/// Supported play modes.
enum GameMode {
    HumanVsHuman,
    HumanVsEngine {
        human_color: Color,
        engine_config: EngineConfig,
    },
    HumanVsStockfish {
        human_color: Color,
        stockfish_config: StockfishConfig,
    },
}

impl GameMode {
    /// User-facing mode label shown in the status header.
    fn label(&self) -> &'static str {
        match self {
            Self::HumanVsHuman => "Human vs Human",
            Self::HumanVsEngine { .. } => "Human vs Engine",
            Self::HumanVsStockfish { .. } => "Human vs Stockfish",
        }
    }

    /// Returns bot color for one-player modes.
    fn engine_color(&self) -> Option<Color> {
        match self {
            Self::HumanVsHuman => None,
            Self::HumanVsEngine { human_color, .. } => Some(match human_color {
                Color::White => Color::Black,
                Color::Black => Color::White,
            }),
            Self::HumanVsStockfish { human_color, .. } => Some(match human_color {
                Color::White => Color::Black,
                Color::Black => Color::White,
            }),
        }
    }

    /// Returns built-in engine config when mode uses the internal engine.
    fn engine_config(&self) -> Option<EngineConfig> {
        match self {
            Self::HumanVsHuman => None,
            Self::HumanVsEngine { engine_config, .. } => Some(*engine_config),
            Self::HumanVsStockfish { .. } => None,
        }
    }

    /// Returns Stockfish config when mode uses the external UCI engine.
    fn stockfish_config(&self) -> Option<StockfishConfig> {
        match self {
            Self::HumanVsHuman | Self::HumanVsEngine { .. } => None,
            Self::HumanVsStockfish {
                stockfish_config, ..
            } => Some(stockfish_config.clone()),
        }
    }
}

/// Prompts the user to choose a game mode.
fn prompt_game_mode() -> io::Result<GameMode> {
    println!("Choose game mode:");
    println!("1) Human vs Human");
    println!("2) Human vs Engine (you play White)");
    println!("3) Human vs Stockfish (you play White)");
    let mut stdout = io::stdout();
    let stdin = io::stdin();

    loop {
        print!("Select mode [1/2/3]: ");
        stdout.flush()?;
        let mut line = String::new();
        stdin.read_line(&mut line)?;
        match line.trim() {
            "1" => return Ok(GameMode::HumanVsHuman),
            "2" => {
                let engine_config = prompt_engine_config()?;
                return Ok(GameMode::HumanVsEngine {
                    human_color: Color::White,
                    engine_config,
                });
            }
            "3" => {
                let stockfish_config = prompt_stockfish_config()?;
                return Ok(GameMode::HumanVsStockfish {
                    human_color: Color::White,
                    stockfish_config,
                });
            }
            _ => {
                println!("Invalid selection. Enter 1, 2, or 3.");
            }
        }
    }
}

/// Prompts for built-in engine strength and timing options.
fn prompt_engine_config() -> io::Result<EngineConfig> {
    let defaults = EngineConfig::default();
    println!();
    println!("Configure engine strength (press Enter to use defaults).");

    let strength = prompt_u8("Strength", 1, 10, defaults.strength)?;
    let min_depth = prompt_u8("Min lookahead depth (plies)", 1, 10, defaults.min_depth)?;
    let max_depth = prompt_u8(
        "Max lookahead depth (plies)",
        min_depth,
        12,
        defaults.max_depth.max(min_depth),
    )?;
    let min_think_ms = prompt_u64(
        "Min think time in ms",
        0,
        60_000,
        defaults.min_think_time.as_millis() as u64,
    )?;
    let max_think_ms = prompt_u64(
        "Max think time in ms",
        min_think_ms,
        120_000,
        defaults
            .max_think_time
            .as_millis()
            .max(min_think_ms as u128) as u64,
    )?;

    let config = EngineConfig {
        strength,
        min_depth,
        max_depth,
        min_think_time: Duration::from_millis(min_think_ms),
        max_think_time: Duration::from_millis(max_think_ms),
    };

    println!("Using engine settings: {}", engine_config_summary(config));
    println!();
    Ok(config)
}

fn prompt_u8(label: &str, min: u8, max: u8, default: u8) -> io::Result<u8> {
    let mut stdout = io::stdout();
    loop {
        print!("{label} [{min}-{max}] (default {default}): ");
        stdout.flush()?;
        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return Ok(default);
        }
        match trimmed.parse::<u8>() {
            Ok(value) if (min..=max).contains(&value) => return Ok(value),
            _ => println!("Please enter a value between {min} and {max}."),
        }
    }
}

/// Prompts for an unsigned integer within a bounded range.
fn prompt_u64(label: &str, min: u64, max: u64, default: u64) -> io::Result<u64> {
    let mut stdout = io::stdout();
    loop {
        print!("{label} [{min}-{max}] (default {default}): ");
        stdout.flush()?;
        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return Ok(default);
        }
        match trimmed.parse::<u64>() {
            Ok(value) if (min..=max).contains(&value) => return Ok(value),
            _ => println!("Please enter a value between {min} and {max}."),
        }
    }
}

/// Prompts for a string value with a default.
fn prompt_string(label: &str, default: &str) -> io::Result<String> {
    let mut stdout = io::stdout();
    print!("{label} (default {default}): ");
    stdout.flush()?;
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    let trimmed = line.trim();
    if trimmed.is_empty() {
        Ok(default.to_string())
    } else {
        Ok(trimmed.to_string())
    }
}

/// Human-readable built-in engine configuration summary.
fn engine_config_summary(config: EngineConfig) -> String {
    format!(
        "Str {} | Depth {}-{} | Think {}-{} ms",
        config.strength,
        config.min_depth,
        config.max_depth,
        config.min_think_time.as_millis(),
        config.max_think_time.as_millis()
    )
}

#[derive(Clone)]
/// Runtime config for an external Stockfish process.
struct StockfishConfig {
    path: String,
    move_time_ms: u64,
    skill_level: u8,
}

/// Stockfish UCI process wrapper.
struct StockfishClient {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    config: StockfishConfig,
}

impl StockfishClient {
    /// Starts and initializes Stockfish via UCI (`uci`, options, `ucinewgame`).
    fn new(config: StockfishConfig) -> io::Result<Self> {
        let mut child = Command::new(&config.path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|err| {
                io::Error::new(err.kind(), format!("failed to start Stockfish: {err}"))
            })?;

        let stdin = child.stdin.take().ok_or_else(|| {
            io::Error::new(io::ErrorKind::BrokenPipe, "failed to open Stockfish stdin")
        })?;
        let stdout = child.stdout.take().ok_or_else(|| {
            io::Error::new(io::ErrorKind::BrokenPipe, "failed to open Stockfish stdout")
        })?;

        let mut client = Self {
            child,
            stdin,
            stdout: BufReader::new(stdout),
            config,
        };
        client.send_line("uci")?;
        client.read_until_prefix("uciok")?;
        client.send_line(&format!(
            "setoption name Skill Level value {}",
            client.config.skill_level
        ))?;
        client.send_line("isready")?;
        client.read_until_prefix("readyok")?;
        client.send_line("ucinewgame")?;
        client.send_line("isready")?;
        client.read_until_prefix("readyok")?;
        Ok(client)
    }

    /// Requests a best move for the provided UCI move history string.
    fn best_move(&mut self, uci_history: &str) -> io::Result<Option<Move>> {
        if uci_history.is_empty() {
            self.send_line("position startpos")?;
        } else {
            self.send_line(&format!("position startpos moves {uci_history}"))?;
        }
        self.send_line(&format!("go movetime {}", self.config.move_time_ms))?;
        loop {
            let line = self.read_line()?;
            if let Some(rest) = line.strip_prefix("bestmove ") {
                let mv_str = rest.split_whitespace().next().unwrap_or("(none)");
                if mv_str == "(none)" {
                    return Ok(None);
                }
                let mv = parse_uci_move(mv_str).ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("invalid Stockfish move: {mv_str}"),
                    )
                })?;
                return Ok(Some(mv));
            }
        }
    }

    fn send_line(&mut self, line: &str) -> io::Result<()> {
        writeln!(self.stdin, "{line}")?;
        self.stdin.flush()
    }

    /// Reads one newline-terminated line from Stockfish stdout.
    fn read_line(&mut self) -> io::Result<String> {
        let mut line = String::new();
        let read = self.stdout.read_line(&mut line)?;
        if read == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Stockfish closed the pipe",
            ));
        }
        Ok(line.trim().to_string())
    }

    /// Reads and discards lines until one starts with `prefix`.
    fn read_until_prefix(&mut self, prefix: &str) -> io::Result<()> {
        loop {
            let line = self.read_line()?;
            if line.starts_with(prefix) {
                return Ok(());
            }
        }
    }
}

impl Drop for StockfishClient {
    fn drop(&mut self) {
        let _ = writeln!(self.stdin, "quit");
        let _ = self.stdin.flush();
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Prompts for Stockfish binary and strength/time parameters.
fn prompt_stockfish_config() -> io::Result<StockfishConfig> {
    println!();
    println!("Configure Stockfish (press Enter to use defaults).");
    let default_path = detect_default_stockfish_path();
    let path = prompt_stockfish_path(&default_path)?;
    let move_time_ms = prompt_u64("Think time per move in ms", 10, 120_000, 1_000)?;
    let skill_level = prompt_u8("Skill level", 0, 20, 12)?;
    println!(
        "Using Stockfish settings: {} | {} ms | skill {}",
        path, move_time_ms, skill_level
    );
    println!();
    Ok(StockfishConfig {
        path,
        move_time_ms,
        skill_level,
    })
}

fn detect_default_stockfish_path() -> String {
    let candidates = [
        "./stockfish/stockfish-ubuntu-x86-64-avx2",
        "./stockfish/stockfish",
        "stockfish",
    ];
    for candidate in candidates {
        if candidate == "stockfish" || Path::new(candidate).is_file() {
            return candidate.to_string();
        }
    }
    "stockfish".to_string()
}

/// Prompts until an executable Stockfish path is provided.
fn prompt_stockfish_path(default: &str) -> io::Result<String> {
    loop {
        let path = prompt_string("Stockfish binary path", default)?;
        match verify_stockfish_path(&path) {
            Ok(()) => return Ok(path),
            Err(err) => println!("Invalid Stockfish path: {err}"),
        }
    }
}

/// Lightweight executability check for a Stockfish binary path.
fn verify_stockfish_path(path: &str) -> io::Result<()> {
    let mut child = Command::new(path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|err| io::Error::new(err.kind(), format!("cannot execute '{path}': {err}")))?;
    let _ = child.kill();
    let _ = child.wait();
    Ok(())
}

#[derive(Default)]
struct CapturedDisplay {
    white_captured_display: String,
    black_captured_display: String,
}

fn eval_line(eval_cp: i32) -> String {
    let side = if eval_cp > 0 {
        "White better"
    } else if eval_cp < 0 {
        "Black better"
    } else {
        "Equal"
    };
    format!("Eval: {:+.2} ({side})", eval_cp as f32 / 100.0)
}

/// Computes captured-piece display strings for both sides.
fn captured_pieces_display(game: &Chess) -> CapturedDisplay {
    let board_counts = board_piece_counts(game);
    let initial = initial_piece_counts();

    let mut white_captured = String::new();
    let mut black_captured = String::new();
    for kind in [
        PieceKind::Queen,
        PieceKind::Rook,
        PieceKind::Bishop,
        PieceKind::Knight,
        PieceKind::Pawn,
    ] {
        let idx = piece_kind_idx(kind);
        let black_missing = initial[idx].saturating_sub(board_counts.black[idx]);
        let white_missing = initial[idx].saturating_sub(board_counts.white[idx]);

        for _ in 0..black_missing {
            white_captured.push(piece_symbol(Color::Black, kind));
            white_captured.push(' ');
        }
        for _ in 0..white_missing {
            black_captured.push(piece_symbol(Color::White, kind));
            black_captured.push(' ');
        }
    }

    CapturedDisplay {
        white_captured_display: if white_captured.is_empty() {
            "-".to_string()
        } else {
            white_captured.trim_end().to_string()
        },
        black_captured_display: if black_captured.is_empty() {
            "-".to_string()
        } else {
            black_captured.trim_end().to_string()
        },
    }
}

/// Formats material balance in pawn units.
fn material_balance_line(balance: i32) -> String {
    if balance > 0 {
        format!("White +{balance}")
    } else if balance < 0 {
        format!("Black +{}", -balance)
    } else {
        "Equal".to_string()
    }
}

fn move_to_uci(mv: Move) -> String {
    let mut text = format!("{}{}", idx_to_square(mv.from), idx_to_square(mv.to));
    if let Some(promotion) = mv.promotion {
        text.push(match promotion {
            PieceKind::Queen => 'q',
            PieceKind::Rook => 'r',
            PieceKind::Bishop => 'b',
            PieceKind::Knight => 'n',
            PieceKind::Pawn | PieceKind::King => 'q',
        });
    }
    text
}

/// Appends a single move to a UCI history string used in `position startpos moves ...`.
fn append_uci_move(uci_history: &mut String, mv: Move) {
    if !uci_history.is_empty() {
        uci_history.push(' ');
    }
    uci_history.push_str(&move_to_uci(mv));
}

fn parse_uci_move(input: &str) -> Option<Move> {
    let bytes = input.as_bytes();
    if bytes.len() < 4 {
        return None;
    }
    let from = parse_uci_square(&input[0..2])?;
    let to = parse_uci_square(&input[2..4])?;
    let promotion = if bytes.len() == 5 {
        match bytes[4] {
            b'q' | b'Q' => Some(PieceKind::Queen),
            b'r' | b'R' => Some(PieceKind::Rook),
            b'b' | b'B' => Some(PieceKind::Bishop),
            b'n' | b'N' => Some(PieceKind::Knight),
            _ => None,
        }
    } else {
        None
    };
    Some(Move {
        from,
        to,
        promotion,
    })
}

/// Parses a UCI square (`e4`) into board index.
fn parse_uci_square(input: &str) -> Option<usize> {
    let bytes = input.as_bytes();
    if bytes.len() != 2 {
        return None;
    }
    let file = match bytes[0] {
        b'a'..=b'h' => (bytes[0] - b'a') as usize,
        _ => return None,
    };
    let rank = match bytes[1] {
        b'1'..=b'8' => (bytes[1] - b'1') as usize,
        _ => return None,
    };
    Some(square_to_idx(file, rank))
}

fn material_balance_pawns(game: &Chess) -> i32 {
    let counts = board_piece_counts(game);
    let mut white = 0i32;
    let mut black = 0i32;
    for kind in [
        PieceKind::Pawn,
        PieceKind::Knight,
        PieceKind::Bishop,
        PieceKind::Rook,
        PieceKind::Queen,
    ] {
        let idx = piece_kind_idx(kind);
        let value = material_value_pawns(kind);
        white += counts.white[idx] as i32 * value;
        black += counts.black[idx] as i32 * value;
    }
    white - black
}

#[derive(Default)]
struct PieceCounts {
    white: [u8; 6],
    black: [u8; 6],
}

fn board_piece_counts(game: &Chess) -> PieceCounts {
    let mut counts = PieceCounts::default();
    for sq in 0..64 {
        if let Some(piece) = game.piece_at(sq) {
            let idx = piece_kind_idx(piece.kind);
            match piece.color {
                Color::White => counts.white[idx] += 1,
                Color::Black => counts.black[idx] += 1,
            }
        }
    }
    counts
}

fn initial_piece_counts() -> [u8; 6] {
    [
        8, // Pawn
        2, // Knight
        2, // Bishop
        2, // Rook
        1, // Queen
        1, // King
    ]
}

fn piece_kind_idx(kind: PieceKind) -> usize {
    match kind {
        PieceKind::Pawn => 0,
        PieceKind::Knight => 1,
        PieceKind::Bishop => 2,
        PieceKind::Rook => 3,
        PieceKind::Queen => 4,
        PieceKind::King => 5,
    }
}

fn material_value_pawns(kind: PieceKind) -> i32 {
    match kind {
        PieceKind::Pawn => 1,
        PieceKind::Knight => 3,
        PieceKind::Bishop => 3,
        PieceKind::Rook => 5,
        PieceKind::Queen => 9,
        PieceKind::King => 0,
    }
}

fn piece_symbol(color: Color, kind: PieceKind) -> char {
    match (color, kind) {
        (Color::White, PieceKind::Pawn) => '♙',
        (Color::White, PieceKind::Knight) => '♘',
        (Color::White, PieceKind::Bishop) => '♗',
        (Color::White, PieceKind::Rook) => '♖',
        (Color::White, PieceKind::Queen) => '♕',
        (Color::White, PieceKind::King) => '♔',
        (Color::Black, PieceKind::Pawn) => '♟',
        (Color::Black, PieceKind::Knight) => '♞',
        (Color::Black, PieceKind::Bishop) => '♝',
        (Color::Black, PieceKind::Rook) => '♜',
        (Color::Black, PieceKind::Queen) => '♛',
        (Color::Black, PieceKind::King) => '♚',
    }
}
