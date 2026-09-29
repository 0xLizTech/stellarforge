//! Pure tick and deviation arithmetic shared by the contract and SDK parity tests.
//!
//! These are free functions (not `#[contractimpl]` methods), so they do not become
//! WASM entry points. `OracleAdapterContract::tick` / `record` call them; the
//! parity table generator calls the same functions so the committed fixture is
//! bound to real contract math rather than a hand copy.

use crate::BPS_DENOMINATOR;

/// Outcome of comparing a proposed price move against a max deviation in bps.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeviationCheck {
    /// `|next - previous| / previous <= max_bps / 10_000`.
    WithinLimit,
    /// Move exceeds the limit (`OracleError::DeviationTooLarge`).
    TooLarge,
    /// An intermediate `checked_mul` overflowed (`OracleError::Overflow`).
    Overflow,
}

/// Rounds `timestamp` down to a multiple of `resolution`.
///
/// Same arithmetic as the private `OracleAdapterContract::tick` path.
#[inline]
pub fn tick_of(timestamp: u64, resolution: u32) -> u64 {
    let resolution = u64::from(resolution);
    timestamp - timestamp % resolution
}

/// Whether a move from `previous` to `next` stays within `max_bps` of `previous`.
///
/// Cross-multiplies so no precision is lost to division:
/// `|next - previous| * 10_000 <= previous * max_bps`.
///
/// Caller must ensure both prices are positive and `max_bps > 0` (the contract
/// enforces those before calling).
#[inline]
pub fn check_price_deviation(previous: i128, next: i128, max_bps: u32) -> DeviationCheck {
    // Both prices are positive at the call site, so subtraction cannot overflow.
    let moved = (next - previous).abs();
    let (Some(lhs), Some(rhs)) = (
        moved.checked_mul(BPS_DENOMINATOR),
        previous.checked_mul(i128::from(max_bps)),
    ) else {
        return DeviationCheck::Overflow;
    };
    if lhs > rhs {
        DeviationCheck::TooLarge
    } else {
        DeviationCheck::WithinLimit
    }
}
