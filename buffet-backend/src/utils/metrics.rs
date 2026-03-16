/// Calculate Sharpe Ratio.
///
/// sharpe = (mean_return - risk_free_rate) / std_dev_return
pub fn calculate_sharpe_ratio(returns: &[f64], risk_free_rate: f64) -> f64 {
    if returns.is_empty() {
        return 0.0;
    }

    let n = returns.len() as f64;
    let mean = returns.iter().sum::<f64>() / n;

    if n < 2.0 {
        return 0.0;
    }

    let variance = returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (n - 1.0);

    let std_dev = variance.sqrt();

    if std_dev == 0.0 {
        return 0.0;
    }

    (mean - risk_free_rate) / std_dev
}

/// Calculate Maximum Drawdown.
pub fn calculate_max_drawdown(equity_curve: &[f64]) -> f64 {
    if equity_curve.is_empty() {
        return 0.0;
    }

    let mut max_drawdown = 0.0;
    let mut peak = equity_curve[0];

    for &value in equity_curve {
        if value > peak {
            peak = value;
        }

        let drawdown = (peak - value) / peak;
        if drawdown > max_drawdown {
            max_drawdown = drawdown;
        }
    }

    max_drawdown
}

/// Calculate Win Rate from a slice of per-trade PnL values.
///
/// win_rate = number of trades with pnl > 0 / total trades
pub fn calculate_win_rate(trades_pnl: &[f64]) -> f64 {
    if trades_pnl.is_empty() {
        return 0.0;
    }
    let wins = trades_pnl.iter().filter(|&&pnl| pnl > 0.0).count();
    wins as f64 / trades_pnl.len() as f64
}

/// Calculate Profit Factor from a slice of per-trade PnL values.
///
/// profit_factor = sum of winning PnLs / abs(sum of losing PnLs)
/// Returns f64::INFINITY if there are winning trades but no losing trades.
/// Returns 0.0 if there are no trades.
pub fn calculate_profit_factor(trades_pnl: &[f64]) -> f64 {
    if trades_pnl.is_empty() {
        return 0.0;
    }
    let gross_profit: f64 = trades_pnl.iter().filter(|&&pnl| pnl > 0.0).sum();
    let gross_loss: f64 = trades_pnl
        .iter()
        .filter(|&&pnl| pnl < 0.0)
        .sum::<f64>()
        .abs();
    if gross_loss == 0.0 {
        if gross_profit > 0.0 {
            f64::INFINITY
        } else {
            0.0
        }
    } else {
        gross_profit / gross_loss
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_drawdown() {
        let equity = vec![100.0, 110.0, 90.0, 120.0, 80.0, 100.0];
        // Peaks: 100, 110, 110, 120, 120, 120
        // Drawdowns: 0, 0, (110-90)/110=0.18, 0, (120-80)/120=0.33, (120-100)/120=0.16
        let mdd = calculate_max_drawdown(&equity);
        assert!((mdd - 0.3333333333).abs() < 1e-6);
    }

    #[test]
    fn test_win_rate() {
        let pnls = vec![100.0, -50.0, 200.0, -30.0, 50.0];
        let wr = calculate_win_rate(&pnls);
        assert!((wr - 0.6).abs() < 1e-9);
    }

    #[test]
    fn test_win_rate_empty() {
        assert_eq!(calculate_win_rate(&[]), 0.0);
    }

    #[test]
    fn test_profit_factor() {
        let pnls = vec![100.0, -50.0, 200.0, -30.0];
        // gross_profit = 300, gross_loss = 80
        let pf = calculate_profit_factor(&pnls);
        assert!((pf - 3.75).abs() < 1e-9);
    }

    #[test]
    fn test_profit_factor_no_losses() {
        let pnls = vec![100.0, 200.0];
        assert_eq!(calculate_profit_factor(&pnls), f64::INFINITY);
    }

    #[test]
    fn test_profit_factor_empty() {
        assert_eq!(calculate_profit_factor(&[]), 0.0);
    }
}
