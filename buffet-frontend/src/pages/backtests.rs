use crate::components::run_backtest_modal::RunBacktestModal;
use crate::components::sidebar::Sidebar;
use crate::routes::Route;
use crate::services::api::{ApiService, BacktestDto};
use crate::theme::COLORS;
use stylist::css;
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(Backtests)]
pub fn backtests() -> Html {
    let backtests = use_state(|| Vec::<BacktestDto>::new());
    let error = use_state(|| None::<String>);
    let show_modal = use_state(|| false);
    let navigator = use_navigator().unwrap();

    // Fetch on mount
    {
        let backtests = backtests.clone();
        let error = error.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                match ApiService::get_backtests().await {
                    Ok(data) => backtests.set(data),
                    Err(e) => error.set(Some(e)),
                }
            });
            || ()
        });
    }

    let on_new_backtest_click = {
        let show_modal = show_modal.clone();
        Callback::from(move |_: MouseEvent| {
            show_modal.set(true);
        })
    };

    let on_modal_cancel = {
        let show_modal = show_modal.clone();
        Callback::from(move |_: ()| {
            show_modal.set(false);
        })
    };

    let on_modal_success = {
        let show_modal = show_modal.clone();
        let navigator = navigator.clone();
        Callback::from(move |bt: BacktestDto| {
            show_modal.set(false);
            navigator.push(&Route::BacktestDetail { id: bt.id });
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

    let b_list = {
        let navigator = navigator.clone();
        backtests.iter().map(|b| {
            let bid = b.id.clone();
            let nav = navigator.clone();
            let id_short = if b.id.len() >= 8 { &b.id[..8] } else { &b.id };
            let sc = status_color(&b.status);

            let on_view_details = Callback::from(move |_: MouseEvent| {
                nav.push(&Route::BacktestDetail { id: bid.clone() });
            });

            html! {
                <div class={css!(
                    r#"
                    background-color: ${surface};
                    border: 1px solid ${border};
                    border-radius: 12px;
                    padding: 20px 24px;
                    display: grid;
                    grid-template-columns: 2fr 1fr 1fr 1fr 1fr 1fr auto;
                    align-items: center;
                    gap: 16px;
                    margin-bottom: 12px;
                    "#,
                    surface = COLORS.surface,
                    border = COLORS.border
                )}>
                    // ID + Symbol
                    <div>
                        <div class={css!("font-weight: 700; font-size: 15px; margin-bottom: 4px;")}>
                            { format!("Backtest {}", id_short) }
                        </div>
                        <div class={css!(
                            "font-size: 12px; color: ${muted};",
                            muted = COLORS.text_muted
                        )}>
                            { format!("Symbol: {}", b.symbol) }
                        </div>
                    </div>

                    // Status
                    <div>
                        <div class={css!(
                            "font-size: 11px; text-transform: uppercase; color: ${muted}; font-weight: 600; margin-bottom: 4px;",
                            muted = COLORS.text_muted
                        )}>
                            { "Status" }
                        </div>
                        <div class={css!(
                            r#"
                            display: inline-flex;
                            align-items: center;
                            gap: 5px;
                            font-size: 12px;
                            font-weight: 700;
                            text-transform: uppercase;
                            color: ${sc};
                            "#,
                            sc = sc
                        )}>
                            <span class={css!(
                                r#"
                                display: inline-block;
                                width: 7px;
                                height: 7px;
                                border-radius: 50%;
                                background-color: ${sc};
                                flex-shrink: 0;
                                "#,
                                sc = sc
                            )}></span>
                            { &b.status }
                        </div>
                    </div>

                    // Return
                    <div>
                        <div class={css!(
                            "font-size: 11px; text-transform: uppercase; color: ${muted}; font-weight: 600; margin-bottom: 4px;",
                            muted = COLORS.text_muted
                        )}>
                            { "Return" }
                        </div>
                        <div class={css!(
                            "font-size: 14px; font-weight: 700; color: ${color};",
                            color = if b.total_return.unwrap_or(0.0) >= 0.0 { COLORS.success } else { COLORS.danger }
                        )}>
                            { format!("{:.2}%", b.total_return.unwrap_or(0.0) * 100.0) }
                        </div>
                    </div>

                    // Sharpe
                    <div>
                        <div class={css!(
                            "font-size: 11px; text-transform: uppercase; color: ${muted}; font-weight: 600; margin-bottom: 4px;",
                            muted = COLORS.text_muted
                        )}>
                            { "Sharpe" }
                        </div>
                        <div class={css!("font-size: 14px; font-weight: 700;")}>
                            { format!("{:.2}", b.sharpe_ratio.unwrap_or(0.0)) }
                        </div>
                    </div>

                    // Trade Count
                    <div>
                        <div class={css!(
                            "font-size: 11px; text-transform: uppercase; color: ${muted}; font-weight: 600; margin-bottom: 4px;",
                            muted = COLORS.text_muted
                        )}>
                            { "Trades" }
                        </div>
                        <div class={css!("font-size: 14px; font-weight: 700;")}>
                            {
                                match b.trade_count {
                                    Some(tc) => tc.to_string(),
                                    None => "—".to_string(),
                                }
                            }
                        </div>
                    </div>

                    // Win Rate
                    <div>
                        <div class={css!(
                            "font-size: 11px; text-transform: uppercase; color: ${muted}; font-weight: 600; margin-bottom: 4px;",
                            muted = COLORS.text_muted
                        )}>
                            { "Win Rate" }
                        </div>
                        <div class={css!(
                            "font-size: 14px; font-weight: 700; color: ${color};",
                            color = if b.win_rate.unwrap_or(0.0) >= 0.5 { COLORS.success } else { COLORS.text_muted }
                        )}>
                            {
                                match b.win_rate {
                                    Some(wr) => format!("{:.1}%", wr * 100.0),
                                    None => "—".to_string(),
                                }
                            }
                        </div>
                    </div>

                    // View Details button
                    <div>
                        <button
                            onclick={on_view_details}
                            class={css!(
                                r#"
                                padding: 7px 14px;
                                border-radius: 8px;
                                border: 1px solid ${border};
                                background: transparent;
                                color: ${text};
                                font-size: 12px;
                                font-weight: 600;
                                cursor: pointer;
                                white-space: nowrap;
                                transition: background 0.15s;
                                "#,
                                border = COLORS.border,
                                text = COLORS.text
                            )}
                        >
                            { "View Details" }
                        </button>
                    </div>
                </div>
            }
        }).collect::<Html>()
    };

    html! {
        <div class={css!(
            "display: flex; min-height: 100vh; background-color: ${bg}; color: white;",
            bg = COLORS.background
        )}>
            <Sidebar />
            <div class={css!("flex: 1; padding: 40px; overflow-y: auto;")}>

                // Header
                <div class={css!("display: flex; justify-content: space-between; align-items: center; margin-bottom: 32px;")}>
                    <h1 class={css!("font-size: 32px; font-weight: 700;")}>{ "Backtests" }</h1>
                    <button
                        onclick={on_new_backtest_click}
                        class={css!(
                            r#"
                            background-color: ${primary};
                            color: white;
                            border: none;
                            padding: 10px 20px;
                            border-radius: 8px;
                            font-size: 14px;
                            font-weight: 600;
                            cursor: pointer;
                            transition: background 0.15s;
                            "#,
                            primary = COLORS.primary
                        )}
                    >
                        { "New Backtest" }
                    </button>
                </div>

                // Error banner
                {
                    if let Some(err) = &*error {
                        html! {
                            <div class={css!(
                                r#"
                                color: ${danger};
                                background: rgba(239, 68, 68, 0.1);
                                border: 1px solid rgba(239, 68, 68, 0.3);
                                border-radius: 8px;
                                padding: 16px;
                                margin-bottom: 24px;
                                font-size: 14px;
                                "#,
                                danger = COLORS.danger
                            )}>
                                { err }
                            </div>
                        }
                    } else {
                        html! {}
                    }
                }

                // Column headers (only when rows exist)
                {
                    if !backtests.is_empty() {
                        html! {
                            <div class={css!(
                                r#"
                                display: grid;
                                grid-template-columns: 2fr 1fr 1fr 1fr 1fr 1fr auto;
                                gap: 16px;
                                padding: 0 24px 10px 24px;
                                "#
                            )}>
                                {
                                    ["Backtest", "Status", "Return", "Sharpe", "Trades", "Win Rate", ""]
                                        .iter()
                                        .map(|h| html! {
                                            <div class={css!(
                                                "font-size: 11px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.06em; color: ${muted};",
                                                muted = COLORS.text_muted
                                            )}>
                                                { *h }
                                            </div>
                                        })
                                        .collect::<Html>()
                                }
                            </div>
                        }
                    } else {
                        html! {}
                    }
                }

                // List or empty state
                {
                    if backtests.is_empty() {
                        html! {
                            <div class={css!(
                                r#"
                                display: flex;
                                flex-direction: column;
                                align-items: center;
                                justify-content: center;
                                padding: 80px 40px;
                                color: ${muted};
                                font-size: 15px;
                                "#,
                                muted = COLORS.text_muted
                            )}>
                                <div class={css!("font-size: 40px; margin-bottom: 16px;")}>{ "🧪" }</div>
                                <div>{ "No backtests found. Run one from a strategy page." }</div>
                            </div>
                        }
                    } else {
                        html! {
                            <div class={css!("display: flex; flex-direction: column;")}>
                                { b_list }
                            </div>
                        }
                    }
                }
            </div>

            // New Backtest modal (no pre-filled strategy_id)
            if *show_modal {
                <RunBacktestModal
                    strategy_id={None}
                    on_success={on_modal_success}
                    on_cancel={on_modal_cancel}
                />
            }
        </div>
    }
}
