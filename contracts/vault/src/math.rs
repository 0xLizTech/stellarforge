//! Pure checked arithmetic used by vault deposit and redemption operations.

/// Returns the fractional shares minted for `amount`, or `None` on i128 overflow.
pub fn shares_for_deposit(amount: i128, exchange_rate: i128) -> Option<i128> {
    amount.checked_mul(exchange_rate)
}

/// Returns underlying units for an exact share multiple, or `None` otherwise.
pub fn underlying_for_redeem(shares: i128, exchange_rate: i128) -> Option<i128> {
    if exchange_rate <= 0 || shares % exchange_rate != 0 {
        return None;
    }
    Some(shares / exchange_rate)
}
