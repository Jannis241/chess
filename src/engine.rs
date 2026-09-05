//! Built-in chess engine used for Human vs Engine mode and UI evaluation.
//!
//! The search is a negamax alpha-beta implementation with iterative deepening,
//! lightweight move ordering, configurable strength, and configurable think-time bounds.

use std::thread;
use std::time::{Duration, Instant};

use crate::game::{Chess, Color, GameStatus, Move, Piece, PieceKind};

const MATE_SCORE: i32 = 100_000;
const CHECK_BONUS: i32 = 35;

#[derive(Clone, Copy, Debug)]
/// Runtime tuning knobs for the built-in engine.
pub struct EngineConfig {
    /// Coarse playing strength from `1` (weakest) to `10` (strongest).
    pub strength: u8,
    /// Minimum iterative-deepening depth (plies).
    pub min_depth: u8,
    /// Maximum iterative-deepening depth (plies).
    pub max_depth: u8,
    /// Minimum wall-clock think time for a move.
    pub min_think_time: Duration,
    /// Maximum wall-clock think time for a move.
    pub max_think_time: Duration,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            strength: 8,
            min_depth: 2,
            max_depth: 4,
            min_think_time: Duration::from_millis(150),
            max_think_time: Duration::from_millis(1_500),
        }
    }
}

#[derive(Clone, Copy, Debug)]
/// Result returned by [`Engine::best_move_timed`].
pub struct EngineMove {
    /// Chosen move.
    pub mv: Move,
    /// Evaluation in centipawns from White's perspective.
    pub eval: i32,
    /// Deepest fully completed depth in plies.
    pub depth_reached: u8,
    /// Elapsed wall-clock time spent searching.
    pub elapsed: Duration,
}

#[derive(Clone, Copy, Debug)]
enum SearchResult {
    Score(i32),
    Timeout,
}

pub struct Engine {
    config: EngineConfig,
}

impl Engine {
    /// Creates a deterministic full-strength engine fixed to one depth.
    pub fn new(max_depth: u8) -> Self {
        Self::with_config(EngineConfig {
            strength: 10,
            min_depth: max_depth.max(1),
            max_depth: max_depth.max(1),
            min_think_time: Duration::ZERO,
            max_think_time: Duration::from_secs(60),
        })
    }

    /// Creates an engine from user-provided config.
    ///
    /// Values are normalized to safe ranges (depth/time/strength clamps).
    pub fn with_config(config: EngineConfig) -> Self {
        let mut cfg = config;
        cfg.strength = cfg.strength.clamp(1, 10);
        cfg.min_depth = cfg.min_depth.max(1);
        cfg.max_depth = cfg.max_depth.max(cfg.min_depth);
        cfg.max_think_time = cfg.max_think_time.max(Duration::from_millis(1));
        cfg.min_think_time = cfg.min_think_time.min(cfg.max_think_time);
        Self { config: cfg }
    }

    /// Returns a static position evaluation in centipawns from White's perspective.
    pub fn evaluate_position(&self, game: &Chess) -> i32 {
        evaluate_for_white(game)
    }

    /// Finds the best move and evaluation, without timing metadata.
    pub fn best_move(&self, game: &Chess) -> Option<(Move, i32)> {
        self.best_move_timed(game).map(|res| (res.mv, res.eval))
    }

    /// Finds a move via iterative deepening within configured depth/time constraints.
    pub fn best_move_timed(&self, game: &Chess) -> Option<EngineMove> {
        let mut root_moves = game.legal_moves_for_turn();
        if root_moves.is_empty() {
            return None;
        }
        order_moves(game, &mut root_moves);

        let start = Instant::now();
        let deadline = start + self.config.max_think_time;
        let required_depth = self.config.min_depth.min(self.config.max_depth).max(1);

        let mut best_completed = None;

        for depth in 1..=self.config.max_depth {
            let enforce_deadline = depth > required_depth;
            if depth > required_depth && Instant::now() >= deadline {
                break;
            }

            let search = self.search_root(game, &root_moves, depth, deadline, enforce_deadline);
            let Some((best_mv, best_eval)) = search else {
                break;
            };

            best_completed = Some((best_mv, best_eval, depth));

            if depth >= required_depth && Instant::now() >= deadline {
                break;
            }
        }

        let (mv, eval, depth_reached) = best_completed?;

        if self.config.min_think_time > Duration::ZERO {
            let elapsed = start.elapsed();
            if elapsed < self.config.min_think_time {
                thread::sleep(self.config.min_think_time - elapsed);
            }
        }

        Some(EngineMove {
            mv,
            eval,
            depth_reached,
            elapsed: start.elapsed(),
        })
    }

    fn search_root(
        &self,
        game: &Chess,
        root_moves: &[Move],
        depth: u8,
        deadline: Instant,
        enforce_deadline: bool,
    ) -> Option<(Move, i32)> {
        let mut alpha = -MATE_SCORE;
        let beta = MATE_SCORE;
        let mut scored = Vec::with_capacity(root_moves.len());
        let mut nodes = 0u64;

        for &mv in root_moves {
            if enforce_deadline && timed_out(deadline) {
                return None;
            }

            let mut next = game.clone();
            if next.try_make_move(mv).is_err() {
                continue;
            }

            let score = match self.search(
                &next,
                depth.saturating_sub(1),
                -beta,
                -alpha,
                1,
                deadline,
                enforce_deadline,
                &mut nodes,
            ) {
                SearchResult::Score(score) => -score,
                SearchResult::Timeout => return None,
            };

            scored.push((mv, score));
            if score > alpha {
                alpha = score;
            }
            if alpha >= beta {
                break;
            }
        }

        if scored.is_empty() {
            return None;
        }

        scored.sort_unstable_by_key(|(_, score)| -*score);
        Some(select_move_by_strength(&scored, self.config.strength))
    }

    fn search(
        &self,
        game: &Chess,
        depth: u8,
        mut alpha: i32,
        beta: i32,
        ply: i32,
        deadline: Instant,
        enforce_deadline: bool,
        nodes: &mut u64,
    ) -> SearchResult {
        *nodes += 1;
        if enforce_deadline && (*nodes & 1023 == 0) && timed_out(deadline) {
            return SearchResult::Timeout;
        }

        match game.status() {
            GameStatus::Checkmate { .. } => return SearchResult::Score(-MATE_SCORE + ply),
            GameStatus::Stalemate
            | GameStatus::DrawFiftyMove
            | GameStatus::DrawThreefoldRepetition
            | GameStatus::DrawInsufficientMaterial => return SearchResult::Score(0),
            GameStatus::Ongoing => {}
        }

        if depth == 0 {
            return SearchResult::Score(eval_for_side_to_move(game));
        }

        let mut moves = game.legal_moves_for_turn();
        if moves.is_empty() {
            return SearchResult::Score(eval_for_side_to_move(game));
        }
        order_moves(game, &mut moves);

        let mut best = -MATE_SCORE;
        for mv in moves {
            if enforce_deadline && (*nodes & 255 == 0) && timed_out(deadline) {
                return SearchResult::Timeout;
            }

            let mut next = game.clone();
            if next.try_make_move(mv).is_err() {
                continue;
            }

            let score = match self.search(
                &next,
                depth - 1,
                -beta,
                -alpha,
                ply + 1,
                deadline,
                enforce_deadline,
                nodes,
            ) {
                SearchResult::Score(score) => -score,
                SearchResult::Timeout => return SearchResult::Timeout,
            };

            if score > best {
                best = score;
            }
            if score > alpha {
                alpha = score;
            }
            if alpha >= beta {
                break;
            }
        }

        SearchResult::Score(best)
    }
}

fn timed_out(deadline: Instant) -> bool {
    Instant::now() >= deadline
}

/// Picks a move from the top score band according to configured strength.
fn select_move_by_strength(scored_moves: &[(Move, i32)], strength: u8) -> (Move, i32) {
    let best = scored_moves[0].1;
    let slack_cp = (10 - strength.clamp(1, 10)) as i32 * 60;
    let threshold = best - slack_cp;

    let mut chosen = scored_moves[0];
    for &(mv, eval) in scored_moves {
        if eval >= threshold {
            chosen = (mv, eval);
        } else {
            break;
        }
    }

    chosen
}

/// Converts White-centric evaluation to side-to-move perspective.
fn eval_for_side_to_move(game: &Chess) -> i32 {
    let white_eval = evaluate_for_white(game);
    match game.current_turn() {
        Color::White => white_eval,
        Color::Black => -white_eval,
    }
}

/// Static evaluation in centipawns from White's perspective.
fn evaluate_for_white(game: &Chess) -> i32 {
    let mut score = 0;
    let mut white_material = 0;
    let mut black_material = 0;

    let mut white_king = None;
    let mut black_king = None;

    for idx in 0..64 {
        let Some(piece) = game.piece_at(idx) else {
            continue;
        };

        let piece_value = material_value(piece.kind);
        let pst = piece_square(piece.kind, piece.color, idx);
        let add = piece_value + pst;

        match piece.color {
            Color::White => {
                score += add;
                white_material += piece_value;
                if piece.kind == PieceKind::King {
                    white_king = Some(idx);
                }
            }
            Color::Black => {
                score -= add;
                black_material += piece_value;
                if piece.kind == PieceKind::King {
                    black_king = Some(idx);
                }
            }
        }
    }

    score += mobility_estimate(game, Color::White);
    score -= mobility_estimate(game, Color::Black);

    if let Some(king_sq) = white_king {
        score += king_safety(game, king_sq, Color::White, white_material + black_material);
        if is_square_attacked(game, king_sq, Color::Black) {
            score -= CHECK_BONUS;
        }
    }
    if let Some(king_sq) = black_king {
        score -= king_safety(game, king_sq, Color::Black, white_material + black_material);
        if is_square_attacked(game, king_sq, Color::White) {
            score += CHECK_BONUS;
        }
    }

    if let Some(enemy_king) = black_king {
        score += king_pressure(game, enemy_king, Color::White);
    }
    if let Some(enemy_king) = white_king {
        score -= king_pressure(game, enemy_king, Color::Black);
    }

    score
}

/// In-place move ordering used by search.
fn order_moves(game: &Chess, moves: &mut [Move]) {
    moves.sort_unstable_by_key(|mv| -move_heuristic(game, *mv));
}

/// Lightweight tactical/positional move score for ordering.
fn move_heuristic(game: &Chess, mv: Move) -> i32 {
    let mut score = 0;
    if let Some(captured) = game.piece_at(mv.to) {
        score += 8 * material_value(captured.kind);
    }
    if let Some(promo) = mv.promotion {
        score += 5 * material_value(promo);
    }

    let to_file = (mv.to % 8) as i32;
    let to_rank = (mv.to / 8) as i32;
    score + (3 - (to_file - 3).abs()) + (3 - (to_rank - 3).abs())
}

fn mobility_estimate(game: &Chess, color: Color) -> i32 {
    let mut moves = 0;
    for from in 0..64 {
        let Some(piece) = game.piece_at(from) else {
            continue;
        };
        if piece.color != color {
            continue;
        }

        moves += count_pseudo_targets(game, from, piece) as i32;
    }

    moves * 2
}

fn king_pressure(game: &Chess, enemy_king: usize, attacker: Color) -> i32 {
    let mut pressure = 0;
    let (kf, kr) = idx_to_file_rank(enemy_king);
    for from in 0..64 {
        let Some(piece) = game.piece_at(from) else {
            continue;
        };
        if piece.color != attacker {
            continue;
        }

        let (ff, fr) = idx_to_file_rank(from);
        let dist = (kf as i32 - ff as i32).abs() + (kr as i32 - fr as i32).abs();
        if dist <= 3 {
            pressure += match piece.kind {
                PieceKind::Queen => 16,
                PieceKind::Rook => 10,
                PieceKind::Bishop | PieceKind::Knight => 7,
                PieceKind::Pawn => 3,
                PieceKind::King => 0,
            };
        }

        if attacks_square(game, from, piece, enemy_king) {
            pressure += 8;
        }
    }
    pressure
}

fn king_safety(game: &Chess, king_sq: usize, color: Color, total_material: i32) -> i32 {
    let (file, rank) = idx_to_file_rank(king_sq);
    let mut score = 0;

    // Encourage sheltering behind pawns in front of the king in middlegames.
    if total_material > 2200 {
        let forward = if color == Color::White { 1 } else { -1 };
        for df in -1..=1 {
            let nf = file as i32 + df;
            let nr = rank as i32 + forward;
            if !(0..8).contains(&nf) || !(0..8).contains(&nr) {
                continue;
            }
            let sq = file_rank_to_idx(nf as usize, nr as usize);
            if game.piece_at(sq)
                == Some(Piece {
                    color,
                    kind: PieceKind::Pawn,
                })
            {
                score += 12;
            } else {
                score -= 10;
            }
        }

        let center_distance = (file as i32 - 3).abs() + (rank as i32 - 3).abs();
        score += center_distance * 2;
    }

    score
}

fn is_square_attacked(game: &Chess, sq: usize, by: Color) -> bool {
    for from in 0..64 {
        let Some(piece) = game.piece_at(from) else {
            continue;
        };
        if piece.color != by {
            continue;
        }
        if attacks_square(game, from, piece, sq) {
            return true;
        }
    }
    false
}

fn attacks_square(game: &Chess, from: usize, piece: Piece, target: usize) -> bool {
    if from == target {
        return false;
    }

    let (file, rank) = idx_to_file_rank(from);
    let (to_file, to_rank) = idx_to_file_rank(target);
    let df = to_file as i32 - file as i32;
    let dr = to_rank as i32 - rank as i32;

    match piece.kind {
        PieceKind::Pawn => {
            let forward = if piece.color == Color::White { 1 } else { -1 };
            dr == forward && df.abs() == 1
        }
        PieceKind::Knight => matches!((df.abs(), dr.abs()), (1, 2) | (2, 1)),
        PieceKind::King => df.abs() <= 1 && dr.abs() <= 1,
        PieceKind::Bishop => slider_attacks(game, from, target, df, dr, true, false),
        PieceKind::Rook => slider_attacks(game, from, target, df, dr, false, true),
        PieceKind::Queen => slider_attacks(game, from, target, df, dr, true, true),
    }
}

fn count_pseudo_targets(game: &Chess, from: usize, piece: Piece) -> usize {
    let mut count = 0usize;
    let (file, rank) = idx_to_file_rank(from);

    match piece.kind {
        PieceKind::Pawn => {
            let forward = if piece.color == Color::White { 1 } else { -1 };
            for df in [-1, 1] {
                let nf = file as i32 + df;
                let nr = rank as i32 + forward;
                if (0..8).contains(&nf) && (0..8).contains(&nr) {
                    let sq = file_rank_to_idx(nf as usize, nr as usize);
                    if game.piece_at(sq).is_none_or(|p| p.color != piece.color) {
                        count += 1;
                    }
                }
            }
        }
        PieceKind::Knight => {
            let jumps = [
                (1, 2),
                (2, 1),
                (2, -1),
                (1, -2),
                (-1, -2),
                (-2, -1),
                (-2, 1),
                (-1, 2),
            ];
            for (df, dr) in jumps {
                let nf = file as i32 + df;
                let nr = rank as i32 + dr;
                if (0..8).contains(&nf) && (0..8).contains(&nr) {
                    let sq = file_rank_to_idx(nf as usize, nr as usize);
                    if game.piece_at(sq).is_none_or(|p| p.color != piece.color) {
                        count += 1;
                    }
                }
            }
        }
        PieceKind::Bishop => {
            count += count_slider_targets(
                game,
                file,
                rank,
                piece.color,
                &[(1, 1), (1, -1), (-1, 1), (-1, -1)],
            );
        }
        PieceKind::Rook => {
            count += count_slider_targets(
                game,
                file,
                rank,
                piece.color,
                &[(1, 0), (-1, 0), (0, 1), (0, -1)],
            );
        }
        PieceKind::Queen => {
            count += count_slider_targets(
                game,
                file,
                rank,
                piece.color,
                &[
                    (1, 1),
                    (1, -1),
                    (-1, 1),
                    (-1, -1),
                    (1, 0),
                    (-1, 0),
                    (0, 1),
                    (0, -1),
                ],
            );
        }
        PieceKind::King => {
            for df in -1..=1 {
                for dr in -1..=1 {
                    if df == 0 && dr == 0 {
                        continue;
                    }
                    let nf = file as i32 + df;
                    let nr = rank as i32 + dr;
                    if (0..8).contains(&nf) && (0..8).contains(&nr) {
                        let sq = file_rank_to_idx(nf as usize, nr as usize);
                        if game.piece_at(sq).is_none_or(|p| p.color != piece.color) {
                            count += 1;
                        }
                    }
                }
            }
        }
    }

    count
}

fn count_slider_targets(
    game: &Chess,
    file: usize,
    rank: usize,
    color: Color,
    dirs: &[(i32, i32)],
) -> usize {
    let mut count = 0usize;
    for &(df, dr) in dirs {
        let mut nf = file as i32 + df;
        let mut nr = rank as i32 + dr;
        while (0..8).contains(&nf) && (0..8).contains(&nr) {
            let sq = file_rank_to_idx(nf as usize, nr as usize);
            if let Some(target) = game.piece_at(sq) {
                if target.color != color {
                    count += 1;
                }
                break;
            }
            count += 1;
            nf += df;
            nr += dr;
        }
    }
    count
}

fn slider_attacks(
    game: &Chess,
    from: usize,
    target: usize,
    df: i32,
    dr: i32,
    allow_diag: bool,
    allow_straight: bool,
) -> bool {
    let abs_df = df.abs();
    let abs_dr = dr.abs();
    let is_diag = abs_df == abs_dr && abs_df != 0;
    let is_straight = (df == 0) ^ (dr == 0);
    if !(allow_diag && is_diag || allow_straight && is_straight) {
        return false;
    }

    let step_f = df.signum();
    let step_r = dr.signum();
    let (mut f, mut r) = idx_to_file_rank(from);
    loop {
        f = (f as i32 + step_f) as usize;
        r = (r as i32 + step_r) as usize;
        let sq = file_rank_to_idx(f, r);
        if sq == target {
            return true;
        }
        if game.piece_at(sq).is_some() {
            return false;
        }
    }
}

fn material_value(kind: PieceKind) -> i32 {
    match kind {
        PieceKind::Pawn => 100,
        PieceKind::Knight => 320,
        PieceKind::Bishop => 330,
        PieceKind::Rook => 500,
        PieceKind::Queen => 900,
        PieceKind::King => 0,
    }
}

fn piece_square(kind: PieceKind, color: Color, idx: usize) -> i32 {
    let (file, rank) = idx_to_file_rank(idx);
    let relative_rank = if color == Color::White {
        rank
    } else {
        7 - rank
    };

    let center_bonus = {
        let f = file as i32;
        let r = relative_rank as i32;
        8 - ((f - 3).abs() + (r - 3).abs())
    };

    match kind {
        PieceKind::Pawn => relative_rank as i32 * 10 + center_bonus,
        PieceKind::Knight => center_bonus * 4,
        PieceKind::Bishop => center_bonus * 3,
        PieceKind::Rook => relative_rank as i32 * 2,
        PieceKind::Queen => center_bonus * 2,
        PieceKind::King => {
            if relative_rank >= 6 {
                10
            } else {
                -center_bonus * 3
            }
        }
    }
}

fn idx_to_file_rank(idx: usize) -> (usize, usize) {
    (idx % 8, idx / 8)
}

fn file_rank_to_idx(file: usize, rank: usize) -> usize {
    rank * 8 + file
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eval_prefers_extra_queen() {
        let winning = Chess::from_fen("4k3/8/8/8/8/8/8/3QK3 w - - 0 1").expect("valid");
        let equal = Chess::from_fen("4k3/8/8/8/8/8/8/4K3 w - - 0 1").expect("valid");
        let engine = Engine::new(2);
        assert!(engine.evaluate_position(&winning) > engine.evaluate_position(&equal));
    }

    #[test]
    fn finds_mate_in_one() {
        let game = Chess::from_fen("7k/8/5KQ1/8/8/8/8/8 w - - 0 1").expect("valid");
        let engine = Engine::new(2);
        let (best, _) = engine.best_move(&game).expect("best move");
        let mut after = game.clone();
        after.try_make_move(best).expect("best move must be legal");
        assert_eq!(
            after.status(),
            GameStatus::Checkmate {
                winner: Color::White
            }
        );
    }
}
