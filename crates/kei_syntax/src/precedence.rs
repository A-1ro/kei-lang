//! 二項演算子の優先順位テーブル(単一情報源, #104)。
//!
//! 優先順位はパーサの再帰下降連鎖(`parser::Parser::parse_level` が辿る
//! [`PrecTier::ORDER`] の順)そのものであり、構文知識としてここに 1 箇所だけ
//! 実装する。`kei_check::contract_expr_text` と `kei_fmt` の正規形整形は
//! ここの [`bin_prec`] を参照し、独自の優先順位テーブルを持たない
//! (#69 / #103 で実際に閾値更新漏れが発生した実績への対応)。
//!
//! `kei_emit` の TS 出力用 `Prec` は JS 文法上 relational > equality という
//! 別体系が必要なため、意図的にここには統合しない(kei_emit 側に残る)。
//! ただし `kei_emit` の「同段が右にネストしたときに 1 段強い優先度で出す」
//! ラダーは、この `PrecTier` と同様に `Ord` 導出した列挙から機械的に
//! 「次に強い段」を計算する形にそろえている。

use crate::ast::BinOp;

/// 二項演算子の優先順位段。列挙の並び順(弱い→強い)が単一情報源であり、
/// パーサの再帰下降連鎖 `parse_implies → parse_or → parse_and → parse_cmp →
/// parse_add → parse_mul` と 1 対 1 対応する。新しい優先度段を増やす場合は
/// ここに variant を追加し、[`bin_prec`] とパーサのトークン対応表を更新する。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PrecTier {
    /// `implies`(右結合、最弱)
    Implies,
    /// `||`
    Or,
    /// `&&`
    And,
    /// `==` `!=` `<` `>` `<=` `>=`(単一階層。JS の relational/equality 分離とは異なる)
    Cmp,
    /// `+` `-`
    Add,
    /// `*` `/` `%`(最強)
    Mul,
}

impl PrecTier {
    /// 弱い順から強い順に並んだ全段。パーサの再帰下降連鎖の順序と一致する
    /// (`parser.rs` の `Parser::parse_level` がこの並びをインデックスで辿る)。
    pub const ORDER: [PrecTier; 6] = [
        PrecTier::Implies,
        PrecTier::Or,
        PrecTier::And,
        PrecTier::Cmp,
        PrecTier::Add,
        PrecTier::Mul,
    ];

    /// 0 始まりの段番号(数値が大きいほど強く結合)。`kei_check` / `kei_fmt`
    /// の `bin_prec` が返してきた `u8` と同じ意味。
    pub fn level(self) -> u8 {
        Self::ORDER
            .iter()
            .position(|&t| t == self)
            .expect("PrecTier::ORDER covers all variants") as u8
    }

    /// 1 段強く結合する隣接段。最強段(`Mul`)なら `None`。
    ///
    /// `Ord` 導出済みの `PrecTier` に対し `ORDER` 内の次要素を機械的に返すだけで、
    /// 「次に強い段は何か」を段ごとに手書きしない(kei_emit の `rhs_min` ラダーが
    /// 参考にする導出方法と同じ形)。
    pub fn next_stronger(self) -> Option<PrecTier> {
        let idx = Self::ORDER.iter().position(|&t| t == self)?;
        Self::ORDER.get(idx + 1).copied()
    }
}

/// 二項演算子の優先順位段を返す(単一情報源)。
///
/// パーサの再帰下降連鎖と完全に一致させること。ズレると
/// suggested_contract / 整形結果が実際の AST と異なる括弧付けになる
/// (#69 / #103 参照)。
pub fn bin_prec(op: BinOp) -> PrecTier {
    match op {
        BinOp::Implies => PrecTier::Implies,
        BinOp::Or => PrecTier::Or,
        BinOp::And => PrecTier::And,
        BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge => PrecTier::Cmp,
        BinOp::Add | BinOp::Sub => PrecTier::Add,
        BinOp::Mul | BinOp::Div | BinOp::Rem => PrecTier::Mul,
    }
}

/// [`bin_prec`] の段番号版(`u8`, 数値が大きいほど強く結合)。
/// `kei_check::contract_expr_text` / `kei_fmt` の従来の `bin_prec(op) as u8` 互換。
pub fn bin_prec_level(op: BinOp) -> u8 {
    bin_prec(op).level()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 各 BinOp の優先順位段が仕様通りであることを検証する。
    #[test]
    fn bin_prec_matches_expected_tier() {
        assert_eq!(bin_prec(BinOp::Implies), PrecTier::Implies);
        assert_eq!(bin_prec(BinOp::Or), PrecTier::Or);
        assert_eq!(bin_prec(BinOp::And), PrecTier::And);
        for op in [
            BinOp::Eq,
            BinOp::Ne,
            BinOp::Lt,
            BinOp::Gt,
            BinOp::Le,
            BinOp::Ge,
        ] {
            assert_eq!(bin_prec(op), PrecTier::Cmp);
        }
        for op in [BinOp::Add, BinOp::Sub] {
            assert_eq!(bin_prec(op), PrecTier::Add);
        }
        for op in [BinOp::Mul, BinOp::Div, BinOp::Rem] {
            assert_eq!(bin_prec(op), PrecTier::Mul);
        }
    }

    /// 段番号は弱い順に 0..=5 で単調増加する(パーサの再帰下降連鎖の深さと一致)。
    #[test]
    fn levels_are_monotonically_increasing_in_order() {
        let levels: Vec<u8> = PrecTier::ORDER.iter().map(|t| t.level()).collect();
        assert_eq!(levels, vec![0, 1, 2, 3, 4, 5]);
    }

    /// 隣接段の導出(`next_stronger`)がパーサの連鎖順序と一致することを検証する。
    #[test]
    fn next_stronger_walks_order_forward() {
        assert_eq!(PrecTier::Implies.next_stronger(), Some(PrecTier::Or));
        assert_eq!(PrecTier::Or.next_stronger(), Some(PrecTier::And));
        assert_eq!(PrecTier::And.next_stronger(), Some(PrecTier::Cmp));
        assert_eq!(PrecTier::Cmp.next_stronger(), Some(PrecTier::Add));
        assert_eq!(PrecTier::Add.next_stronger(), Some(PrecTier::Mul));
        assert_eq!(PrecTier::Mul.next_stronger(), None);
    }

    /// `bin_prec_level` は `bin_prec(op).level()` と一致する(fmt/check 互換の入口)。
    #[test]
    fn bin_prec_level_matches_tier_level() {
        for op in [
            BinOp::Implies,
            BinOp::Or,
            BinOp::And,
            BinOp::Eq,
            BinOp::Ne,
            BinOp::Lt,
            BinOp::Gt,
            BinOp::Le,
            BinOp::Ge,
            BinOp::Add,
            BinOp::Sub,
            BinOp::Mul,
            BinOp::Div,
            BinOp::Rem,
        ] {
            assert_eq!(bin_prec_level(op), bin_prec(op).level());
        }
    }
}
