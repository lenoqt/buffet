use crate::components::sidebar::Sidebar;
use crate::services::api::{ApiService, BacktestDto};
use crate::theme::COLORS;
use stylist::css;
use yew::prelude::*;

#[function_component(Backtests)]
pub fn backtests() -> Html {
    let backtests = use_state(|| Vec::<BacktestDto>::new());
    let error = use_state(|| None::<String>);

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

    let b_list = backtests.iter().map(|b| {
        html! {
            <div class={css!(
                r#"
                background-color: ${surface};
                border: 1px solid ${border};
                border-radius: 12px;
                padding: 20px;
                display: grid;
                grid-template-columns: 2fr 1fr 1fr 1fr 1fr;
                align-items: center;
                margin-bottom: 16px;
                "#,
                surface = COLORS.surface,
                border = COLORS.border
            )}>
                <div>
                    <div class={css!("font-weight: 700; font-size: 16px; margin-bottom: 4px;")}>{ format!("Backtest {}", &b.id[..8]) }</div>
                    <div class={css!("font-size: 12px; color: ${text_muted};", text_muted = COLORS.text_muted)}>{ format!("Symbol: {}", b.symbol) }</div>
                </div>

                <div>
                    <div class={css!("font-size: 11px; text-transform: uppercase; color: ${text_muted}; font-weight: 600; margin-bottom: 4px;", text_muted = COLORS.text_muted)}>{ "Status" }</div>
                    <div class={css!(
                        r#"
                        font-size: 13px;
                        font-weight: 600;
                        color: ${color};
                        "#,
                        color = if b.status == "completed" { COLORS.success } else if b.status == "failed" { COLORS.danger } else { COLORS.primary }
                    )}>{ b.status.clone() }</div>
                </div>

                <div>
                    <div class={css!("font-size: 11px; text-transform: uppercase; color: ${text_muted}; font-weight: 600; margin-bottom: 4px;", text_muted = COLORS.text_muted)}>{ "Return" }</div>
                    <div class={css!(
                        r#"
                        font-size: 13px;
                        font-weight: 600;
                        color: ${color};
                        "#,
                        color = if b.total_return.unwrap_or(0.0) >= 0.0 { COLORS.success } else { COLORS.danger }
                    )}>{ format!("{:.2}%", b.total_return.unwrap_or(0.0) * 100.0) }</div>
                </div>

                <div>
                    <div class={css!("font-size: 11px; text-transform: uppercase; color: ${text_muted}; font-weight: 600; margin-bottom: 4px;", text_muted = COLORS.text_muted)}>{ "Sharpe" }</div>
                    <div class={css!("font-size: 13px; font-weight: 600;")}>{ format!("{:.2}", b.sharpe_ratio.unwrap_or(0.0)) }</div>
                </div>

                <div style="text-align: right;">
                    <button class={css!("padding: 6px 12px; border-radius: 6px; border: 1px solid ${border}; background: transparent; color: white; cursor: pointer; font-size: 12px; font-weight: 600;", border = COLORS.border)}>{ "View Details" }</button>
                </div>
            </div>
        }
    }).collect::<Html>();

    html! {
        <div class={css!("display: flex; min-height: 100vh; background-color: ${bg}; color: white;", bg = COLORS.background)}>
            <Sidebar />
            <div class={css!("flex: 1; padding: 40px; overflow-y: auto;")}>
                <div class={css!("display: flex; justify-content: space-between; align-items: center; margin-bottom: 32px;")}>
                    <h1 class={css!("font-size: 32px; font-weight: 700;")}>{ "Backtests" }</h1>
                    <button class={css!(
                        r#"
                        background-color: ${primary};
                        color: white;
                        border: none;
                        padding: 10px 20px;
                        border-radius: 8px;
                        font-weight: 600;
                        cursor: pointer;
                        "#,
                        primary = COLORS.primary
                    )}>
                        { "New Backtest" }
                    </button>
                </div>

                {
                    if let Some(err) = &*error {
                        html! { <div class={css!("color: ${danger}; padding: 16px; background: rgba(239, 68, 68, 0.1); border-radius: 8px; margin-bottom: 24px;", danger = COLORS.danger)}>{ err }</div> }
                    } else if backtests.is_empty() {
                        html! { <div class={css!("color: ${text_muted};", text_muted = COLORS.text_muted)}>{ "No backtests found. Run one from a strategy page." }</div> }
                    } else {
                        html! {
                            <div class={css!("display: flex; flex-direction: column;")}>
                                { b_list }
                            </div>
                        }
                    }
                }
            </div>
        </div>
    }
}
