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
}
