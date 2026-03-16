use crate::components::loading::Spinner;
use crate::components::sidebar::Sidebar;
use crate::routes::Route;
use crate::services::api::{ApiService, BacktestDto, BacktestTradeDto};
use crate::theme::COLORS;
use stylist::css;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq)]
pub struct BacktestDetailProps {
    pub id: String,
}

#[function_component(BacktestDetail)]
pub fn backtest_detail(props: &BacktestDetailProps) -> Html {
    let backtest = use_state(|| None::<BacktestDto>);
    let trades = use_state(|| Vec::<BacktestTradeDto>::new());
    let loading = use_state(|| true);
    let trades_loading = use_state(|| true);
    let error = use_state(|| None::<String>);
    let trades_error = use_state(|| None::<String>);
    let navigator = use_navigator().unwrap();

    // Fetch backtest details
    {
        let backtest = backtest.clone();
        let loading = loading.clone();
        let error = error.clone();
        let id = props.id.clone();
        use_effect_with(id.clone(), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                match ApiService::get_backtest(&id).await {
                    Ok(data) => {
                        backtest.set(Some(data));
                        loading.set(false);
                    }
                    Err(e) => {
                        error.set(Some(e));
                        loading.set(false);
                    }
                }
            });
            || ()
        });
    }

    // Fetch trades
    {
        let trades = trades.clone();
        let trades_loading = trades_loading.clone();
        let trades_error = trades_error.clone();
        let id = props.id.clone();
        use_effect_with(id.clone(), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                match ApiService::get_backtest_trades(&id).await {
                    Ok(data) => {
                        trades.set(data);
                        trades_loading.set(false);
                    }
                    Err(e) => {
                        trades_error.set(Some(e));
                        trades_loading.set(false);
                    }
                }
            });
            || ()
        });
    }

    let on_back = {
        let navigator = navigator.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            navigator.push(&Route::Backtests);
        })
    };

    let status_color = |status: &str| -> &'static str {
        match status {
            "completed" => COLORS.success,
            "failed" => COLORS.danger,
            "running" => COLORS.secondary,
            _ => COLORS.text_muted,
        }
    };

    let card_css = css!(
        r#"
        background-color: ${surface};
        border: 1px solid ${border};
        border-radius: 14px;
        padding: 24px;
        margin-bottom: 24px;
        "#,
        surface = COLORS.surface,
        border = COLORS.border
    );

    let section_label = css!(
        r#"
        font-size: 11px;
        font-weight: 700;
        text-transform: uppercase;
        letter-spacing: 0.08em;
        color: ${muted};
        margin-bottom: 16px;
        "#,
        muted = COLORS.text_muted
    );

    let metric_label_css = css!(
        r#"
        font-size: 11px;
        text-transform: uppercase;
        font-weight: 600;
        color: ${muted};
        margin-bottom: 6px;
        letter-spacing: 0.05em;
        "#,
        muted = COLORS.text_muted
    );

    let metric_value_css = css!(
        r#"
        font-size: 22px;
        font-weight: 700;
        color: ${text};
        "#,
        text = COLORS.text
    );

    html! {
        <div class={css!(
            "display: flex; min-height: 100vh; background-color: ${bg}; color: white;",
            bg = COLORS.background
        )}>
            <Sidebar />
            <div class={css!("flex: 1; padding: 40px; overflow-y: auto;")}>

                // Back link
                <button
                    onclick={on_back}
                    class={css!(
                        r#"
                        display: inline-flex;
                        align-items: center;
                        gap: 8px;
                        background: transparent;
                        border: none;
                        color: ${muted};
                        font-size: 14px;
                        font-weight: 500;
                        cursor: pointer;
                        padding: 0;
                        margin-bottom: 32px;
                        "#,
                        muted = COLORS.text_muted
                    )}
                >
                    { "← Back to Backtests" }
                </button>

                {
                    if *loading {
                        html! { <Spinner /> }
                    } else if let Some(err) = &*error {
                        html! {
                            <div class={css!(
                                r#"
                                color: ${danger};
                                background: rgba(239, 68, 68, 0.1);
                                border: 1px solid rgba(239, 68, 68, 0.3);
                                border-radius: 10px;
                                padding: 20px;
                                "#,
                                danger = COLORS.danger
                            )}>
                                { format!("Failed to load backtest: {}", err) }
                            </div>
                        }
                    } else if let Some(b) = &*backtest {
                        let id_short = if b.id.len() >= 8 { &b.id[..8] } else { &b.id };
                        let sc = status_color(&b.status);
                        html! {
                            <>
                                // Header
                                <div class={css!("margin-bottom: 32px;")}>
                                    <div class={css!("display: flex; align-items: center; gap: 16px; margin-bottom: 8px;")}>
                                        <h1 class={css!("font-size: 28px; font-weight: 700;")}>
                                            { format!("Backtest {}", id_short) }
                                        </h1>
                                        <span class={css!(
                                            r#"
                                            padding: 4px 14px;
                                            border-radius: 999px;
                                            font-size: 12px;
                                            font-weight: 700;
                                            text-transform: uppercase;
                                            background: rgba(0,0,0,0.3);
                                            color: ${sc};
                                            border: 1px solid ${sc};
                                            "#,
                                            sc = sc
                                        )}>
                                            { &b.status }
                                        </span>
                                    </div>
                                    <div class={css!("font-size: 14px; color: ${muted};", muted = COLORS.text_muted)}>
                                        { format!("Symbol: {} · Strategy: {}", b.symbol, &b.strategy_id[..8.min(b.strategy_id.len())]) }
                                    </div>
                                </div>

                                // Metrics grid
                                <div class={card_css.clone()}>
                                    <div class={section_label.clone()}>{ "Performance Metrics" }</div>
                                    <div class={css!("display: grid; grid-template-columns: repeat(auto-fill, minmax(160px, 1fr)); gap: 24px;")}>

                                        // Initial Balance
                                        <div>
                                            <div class={metric_label_css.clone()}>{ "Initial Balance" }</div>
                                            <div class={metric_value_css.clone()}>
                                                { format!("${:.2}", b.initial_balance) }
                                            </div>
                                        </div>

                                        // Final Balance
                                        <div>
                                            <div class={metric_label_css.clone()}>{ "Final Balance" }</div>
                                            <div class={css!(
                                                "font-size: 22px; font-weight: 700; color: ${color};",
                                                color = if b.final_balance.unwrap_or(0.0) >= b.initial_balance { COLORS.success } else { COLORS.danger }
                                            )}>
                                                { format!("${:.2}", b.final_balance.unwrap_or(0.0)) }
                                            </div>
                                        </div>

                                        // Total Return
                                        <div>
                                            <div class={metric_label_css.clone()}>{ "Total Return" }</div>
                                            <div class={css!(
                                                "font-size: 22px; font-weight: 700; color: ${color};",
                                                color = if b.total_return.unwrap_or(0.0) >= 0.0 { COLORS.success } else { COLORS.danger }
                                            )}>
                                                { format!("{:.2}%", b.total_return.unwrap_or(0.0) * 100.0) }
                                            </div>
                                        </div>

                                        // Sharpe Ratio
                                        <div>
                                            <div class={metric_label_css.clone()}>{ "Sharpe Ratio" }</div>
                                            <div class={metric_value_css.clone()}>
                                                { format!("{:.3}", b.sharpe_ratio.unwrap_or(0.0)) }
                                            </div>
                                        </div>

                                        // Max Drawdown
                                        <div>
                                            <div class={metric_label_css.clone()}>{ "Max Drawdown" }</div>
                                            <div class={css!("font-size: 22px; font-weight: 700; color: ${danger};", danger = COLORS.danger)}>
                                                { format!("{:.2}%", b.max_drawdown.unwrap_or(0.0) * 100.0) }
                                            </div>
                                        </div>

                                        // Win Rate
                                        <div>
                                            <div class={metric_label_css.clone()}>{ "Win Rate" }</div>
                                            <div class={css!(
                                                "font-size: 22px; font-weight: 700; color: ${color};",
                                                color = if b.win_rate.unwrap_or(0.0) >= 0.5 { COLORS.success } else { COLORS.danger }
                                            )}>
                                                {
                                                    match b.win_rate {
                                                        Some(wr) => format!("{:.1}%", wr * 100.0),
                                                        None => "—".to_string(),
                                                    }
                                                }
                                            </div>
                                        </div>

                                        // Profit Factor
                                        <div>
                                            <div class={metric_label_css.clone()}>{ "Profit Factor" }</div>
                                            <div class={metric_value_css.clone()}>
                                                {
                                                    match b.profit_factor {
                                                        Some(pf) => format!("{:.2}", pf),
                                                        None => "—".to_string(),
                                                    }
                                                }
                                            </div>
                                        </div>

                                        // Trade Count
                                        <div>
                                            <div class={metric_label_css.clone()}>{ "Trade Count" }</div>
                                            <div class={metric_value_css.clone()}>
                                                {
                                                    match b.trade_count {
                                                        Some(tc) => tc.to_string(),
                                                        None => "—".to_string(),
                                                    }
                                                }
                                            </div>
                                        </div>

                                    </div>
                                </div>

                                // Run Config card
                                <div class={card_css.clone()}>
                                    <div class={section_label.clone()}>{ "Run Configuration" }</div>
                                    <div class={css!("display: grid; grid-template-columns: repeat(auto-fill, minmax(180px, 1fr)); gap: 20px;")}>
                                        <div>
                                            <div class={metric_label_css.clone()}>{ "Commission Rate" }</div>
                                            <div class={css!("font-size: 16px; font-weight: 600;")}>
                                                { format!("{:.4} ({:.2}%)", b.commission_rate, b.commission_rate * 100.0) }
                                            </div>
                                        </div>
                                        <div>
                                            <div class={metric_label_css.clone()}>{ "Slippage (BPS)" }</div>
                                            <div class={css!("font-size: 16px; font-weight: 600;")}>
                                                { format!("{:.1} bps", b.slippage_bps) }
                                            </div>
                                        </div>
                                        if let Some(rc) = &b.run_config {
                                            <div class={css!("grid-column: 1 / -1;")}>
                                                <div class={metric_label_css.clone()}>{ "Run Config" }</div>
                                                <pre class={css!(
                                                    r#"
                                                    font-family: 'JetBrains Mono', 'Fira Code', monospace;
                                                    font-size: 12px;
                                                    color: ${text};
                                                    background: ${bg};
                                                    border-radius: 8px;
                                                    padding: 12px;
                                                    margin: 0;
                                                    overflow-x: auto;
                                                    white-space: pre-wrap;
                                                    word-break: break-word;
                                                    "#,
                                                    text = COLORS.text,
                                                    bg = COLORS.background
                                                )}>
                                                    {
                                                        serde_json::from_str::<serde_json::Value>(rc)
                                                            .map(|v| serde_json::to_string_pretty(&v).unwrap_or_else(|_| rc.clone()))
                                                            .unwrap_or_else(|_| rc.clone())
                                                    }
                                                </pre>
                                            </div>
                                        }
                                    </div>
                                </div>

                                // Trades table
                                <div class={card_css.clone()}>
                                    <div class={section_label.clone()}>{ "Trades" }</div>
                                    {
                                        if *trades_loading {
                                            html! { <Spinner /> }
                                        } else if let Some(terr) = &*trades_error {
                                            html! {
                                                <div class={css!("color: ${danger}; font-size: 14px;", danger = COLORS.danger)}>
                                                    { format!("Failed to load trades: {}", terr) }
                                                </div>
                                            }
                                        } else if trades.is_empty() {
                                            html! {
                                                <div class={css!("color: ${muted}; font-size: 14px; padding: 24px 0; text-align: center;", muted = COLORS.text_muted)}>
                                                    { "No trades found for this backtest." }
                                                </div>
                                            }
                                        } else {
                                            html! {
                                                <div class={css!("overflow-x: auto;")}>
                                                    <table class={css!(
                                                        r#"
                                                        width: 100%;
                                                        border-collapse: collapse;
                                                        font-size: 13px;
                                                        "#
                                                    )}>
                                                        <thead>
                                                            <tr>
                                                                {
                                                                    ["Side", "Entry Price", "Exit Price", "Entry Time", "Exit Time", "PnL", "% Return"]
                                                                        .iter()
                                                                        .map(|h| html! {
                                                                            <th class={css!(
                                                                                r#"
                                                                                text-align: left;
                                                                                padding: 10px 14px;
                                                                                font-size: 11px;
                                                                                font-weight: 700;
                                                                                text-transform: uppercase;
                                                                                letter-spacing: 0.06em;
                                                                                color: ${muted};
                                                                                border-bottom: 1px solid ${border};
                                                                                white-space: nowrap;
                                                                                "#,
                                                                                muted = COLORS.text_muted,
                                                                                border = COLORS.border
                                                                            )}>
                                                                                { *h }
                                                                            </th>
                                                                        })
                                                                        .collect::<Html>()
                                                                }
                                                            </tr>
                                                        </thead>
                                                        <tbody>
                                                            {
                                                                trades.iter().map(|t| {
                                                                    let pnl = t.pnl.unwrap_or(0.0);
                                                                    let pct = t.percentage_return.unwrap_or(0.0);
                                                                    let pnl_color = if pnl >= 0.0 { COLORS.success } else { COLORS.danger };
                                                                    let pct_color = if pct >= 0.0 { COLORS.success } else { COLORS.danger };
                                                                    let side_color = if t.side.to_lowercase() == "long" || t.side.to_lowercase() == "buy" {
                                                                        COLORS.success
                                                                    } else {
                                                                        COLORS.danger
                                                                    };

                                                                    let td_css = css!(
                                                                        r#"
                                                                        padding: 12px 14px;
                                                                        border-bottom: 1px solid ${border};
                                                                        white-space: nowrap;
                                                                        "#,
                                                                        border = COLORS.border
                                                                    );

                                                                    html! {
                                                                        <tr class={css!(
                                                                            r#"
                                                                            transition: background 0.15s;
                                                                            "#
                                                                        )}>
                                                                            // Side
                                                                            <td class={td_css.clone()}>
                                                                                <span class={css!(
                                                                                    r#"
                                                                                    font-weight: 700;
                                                                                    text-transform: uppercase;
                                                                                    font-size: 12px;
                                                                                    color: ${color};
                                                                                    "#,
                                                                                    color = side_color
                                                                                )}>
                                                                                    { &t.side }
                                                                                </span>
                                                                            </td>
                                                                            // Entry Price
                                                                            <td class={td_css.clone()}>
                                                                                { format!("${:.4}", t.entry_price) }
                                                                            </td>
                                                                            // Exit Price
                                                                            <td class={td_css.clone()}>
                                                                                {
                                                                                    match t.exit_price {
                                                                                        Some(ep) => format!("${:.4}", ep),
                                                                                        None => "—".to_string(),
                                                                                    }
                                                                                }
                                                                            </td>
                                                                            // Entry Time
                                                                            <td class={td_css.clone()}>
                                                                                <span class={css!("font-family: monospace; font-size: 12px;")}>
                                                                                    { &t.entry_time }
                                                                                </span>
                                                                            </td>
                                                                            // Exit Time
                                                                            <td class={td_css.clone()}>
                                                                                <span class={css!("font-family: monospace; font-size: 12px;")}>
                                                                                    {
                                                                                        match &t.exit_time {
                                                                                            Some(et) => et.as_str(),
                                                                                            None => "—",
                                                                                        }
                                                                                    }
                                                                                </span>
                                                                            </td>
                                                                            // PnL
                                                                            <td class={td_css.clone()}>
                                                                                <span class={css!(
                                                                                    "font-weight: 600; color: ${color};",
                                                                                    color = pnl_color
                                                                                )}>
                                                                                    {
                                                                                        match t.pnl {
                                                                                            Some(v) => format!("${:.2}", v),
                                                                                            None => "—".to_string(),
                                                                                        }
                                                                                    }
                                                                                </span>
                                                                            </td>
                                                                            // % Return
                                                                            <td class={td_css.clone()}>
                                                                                <span class={css!(
                                                                                    "font-weight: 600; color: ${color};",
                                                                                    color = pct_color
                                                                                )}>
                                                                                    {
                                                                                        match t.percentage_return {
                                                                                            Some(v) => format!("{:.2}%", v * 100.0),
                                                                                            None => "—".to_string(),
                                                                                        }
                                                                                    }
                                                                                </span>
                                                                            </td>
                                                                        </tr>
                                                                    }
                                                                }).collect::<Html>()
                                                            }
                                                        </tbody>
                                                    </table>
                                                </div>
                                            }
                                        }
                                    }
                                </div>
                            </>
                        }
                    } else {
                        html! {
                            <div class={css!("color: ${muted};", muted = COLORS.text_muted)}>
                                { "Backtest not found." }
                            </div>
                        }
                    }
                }
            </div>
        </div>
    }
}
