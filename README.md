# Terminal Chess (Rust)

A fully playable terminal chess game written in Rust, with:

- Complete legal move generation (including castling, en passant, promotion)
- Game-end detection (checkmate, stalemate, fifty-move rule, threefold repetition, insufficient material)
- Human vs Human, Human vs built-in Engine, and Human vs Stockfish modes
- A built-in alpha-beta engine with static evaluation and move ordering
- Optional UCI Stockfish integration
- Last-move highlighting and visible bot thinking state in the UI
- Cached UI analysis data for smoother cursor navigation and redraws

## Quick Start

### Requirements

- Rust toolchain (stable) with `cargo`
- A terminal that supports ANSI colors and Unicode chess symbols
- Optional: Stockfish binary in `PATH` (or provide a custom binary path in the game menu)
  - The game auto-detects `./stockfish/stockfish-ubuntu-x86-64-avx2` when present

### Build and Run

```bash
cargo run
```

You will be prompted for game mode:

1. Human vs Human
2. Human vs Engine (you play White, with configurable strength/depth/time)
3. Human vs Stockfish (you play White, with configurable path/time/skill)

### Stockfish Path Resolution

When choosing mode 3, the default Stockfish path is auto-detected in this order:

1. `./stockfish/stockfish-ubuntu-x86-64-avx2`
2. `./stockfish/stockfish`
3. `stockfish` (from `PATH`)

The prompt validates executability and re-prompts if the path is invalid.

### Controls

- Arrow keys: move cursor
- Enter: select piece / confirm move
- Esc: clear current selection
- `q`: quit

## UI Highlights

- Selected square and cursor are highlighted
- Legal targets from a selected piece are highlighted
- Checked kings are highlighted
- The most recent move (`from` and `to`) is highlighted
- During bot turns, the status line shows `Engine is thinking...` or `Stockfish is thinking...`

## Project Structure

- `src/main.rs`: terminal UI, game loop, input handling, mode selection, Stockfish/UCI process integration
- `src/game.rs`: chess rules, board state, move generation, legality checks, draw/checkmate logic
- `src/engine.rs`: search + evaluation engine

## Runtime Architecture

- `main.rs` coordinates input, drawing, and active mode turn execution
- `game.rs` is the single source of truth for legality and move application
- `engine.rs` is a pure in-process search/evaluation engine
- Stockfish mode is external-process UCI orchestration layered on top of the same `Chess` state

## Engine Overview

The engine uses:

- Negamax with alpha-beta pruning
- Iterative deepening between configurable min/max depth
- Configurable min/max think time per move
- Configurable strength level (1-10)
- Move ordering using a lightweight heuristic:
  - Captures
  - Promotions
  - Centralization bonus
- Static evaluation composed of:
  - Material
  - Piece-square style bonus
  - Mobility estimate
  - King safety and king pressure
  - Check bonus/penalty

## Rules Coverage

Implemented:

- Legal move filtering via king-safety validation
- Castling rights updates
- En passant legality and capture handling
- Promotion (UI move fallback favors queen when promotion piece is omitted)
- Draws by:
  - Fifty-move rule
  - Threefold repetition (position key includes side to move, castling, effective en passant)
  - Insufficient material

## Development

### Run Tests

```bash
cargo test
```

Current tests cover:

- Initial legal move count
- En passant
- Castling
- Promotion
- Checkmate and stalemate detection
- Insufficient material
- Threefold repetition
- Engine sanity checks (material preference + mate in one)

### Formatting

```bash
cargo fmt
```

## Performance Notes

Recent optimizations in this codebase include:

- Reduced allocation and linear-scan overhead in board rendering highlights
- Faster status evaluation path for checking whether any legal move exists
- Lower-overhead move matching in `try_make_move`
- In-place move ordering in search
- Allocation-free attack and mobility calculations in evaluation
- Cached game analytics (`status`, eval, captures, material, checked kings) refreshed only after moves
- Incremental UCI move history string for Stockfish (avoids per-turn `join()` allocations)
- Lower redraw overhead by avoiding full-board newline replacement copies

These improvements make both UI responsiveness and engine search noticeably faster under the same search depth.

## Limitations

- No transposition table
- No quiescence search
- Stockfish mode requires a locally available Stockfish binary

## Suggested Next Steps

- Add a transposition table (Zobrist hashing)
- Add quiescence search for tactical stability near leaves
- Add perft tests for move generator validation depth-by-depth
- Add a FEN loader option in the CLI for debugging positions
