# chess

Chess in the terminal, written in Rust. You can play with the arrow keys, and the game shows you all legal moves of the piece you selected.

## Features

- Legal move generation for all pieces
- Moves that would leave your own king in check are not allowed
- Castling (king side and queen side)
- The game ends when a player has no legal moves left
- Start from any position with a FEN string
- Legal moves of the selected piece are highlighted

**Not done yet:** pawn promotion, en passant, and telling the difference between checkmate and stalemate.

## Run

```sh
cargo run --release
```

## Controls

| Key | Action |
|---|---|
| Arrow keys | move the cursor |
| Enter | select a piece / move it to the cursor |

The start position is set in `src/main.rs` with `board.setup("...")`. You can put any FEN string there.

## Project structure

```
src/chess.rs    board, pieces, move generation, castling, FEN
src/visuals.rs  drawing the board in the terminal
src/input.rs    reading the keyboard input (crossterm)
src/main.rs     game loop
```

## What I learned
- How to represent a chess board and generate moves for each piece
- How to check if a move leaves the king in check
- How to read keyboard input directly in the terminal with crossterm
- This is one of my earlier projects, i built it because i enjoyed chess at the time and thought it would be an interesting learning project.
