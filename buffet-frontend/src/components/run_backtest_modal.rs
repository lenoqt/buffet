use crate::services::api::{ApiService, BacktestDto, RunBacktestDto};
use crate::theme::COLORS;
use stylist::css;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct RunBacktestModalProps {
    pub strategy_id: Option<String>,
    pub on_success: Callback<BacktestDto>,
    pub on_cancel: Callback<()>,
}

#[function_component(RunBacktestModal)]
pub fn run_backtest_modal(props: &RunBacktestModalProps) -> Html {
    let strategy_id = use_state(|| props.strategy_id.clone().unwrap_or_default());
    let symbol = use_state(|| String::new());
    let start_date = use_state(|| String::new());
    let end_date = use_state(|| String::new());
    let initial_balance = use_state(|| "10000".to_string());
    let commission_rate = use_state(|| "0.001".to_string());
    let slippage_bps = use_state(|| "10".to_string());
    let loading = use_state(|| false);
    let error = use_state(|| None::<String>);

    let on_strategy_id_input = {
        let strategy_id = strategy_id.clone();
        Callback::from(move |e: InputEvent| {
            let input = e.target_unchecked_into::<web_sys::HtmlInputElement>();
            strategy_id.set(input.value());
        })
    };

    let on_symbol_input = {
        let symbol = symbol.clone();
        Callback::from(move |e: InputEvent| {
            let input = e.target_unchecked_into::<web_sys::HtmlInputElement>();
            symbol.set(input.value());
        })
    };

    let on_start_date_input = {
        let start_date = start_date.clone();
        Callback::from(move |e: InputEvent| {
            let input = e.target_unchecked_into::<web_sys::HtmlInputElement>();
            start_date.set(input.value());
        })
    };

    let on_end_date_input = {
        let end_date = end_date.clone();
        Callback::from(move |e: InputEvent| {
            let input = e.target_unchecked_into::<web_sys::HtmlInputElement>();
            end_date.set(input.value());
        })
    };

    let on_balance_input = {
        let initial_balance = initial_balance.clone();
        Callback::from(move |e: InputEvent| {
            let input = e.target_unchecked_into::<web_sys::HtmlInputElement>();
            initial_balance.set(input.value());
        })
    };

    let on_commission_input = {
        let commission_rate = commission_rate.clone();
        Callback::from(move |e: InputEvent| {
            let input = e.target_unchecked_into::<web_sys::HtmlInputElement>();
            commission_rate.set(input.value());
        })
    };

    let on_slippage_input = {
        let slippage_bps = slippage_bps.clone();
        Callback::from(move |e: InputEvent| {
            let input = e.target_unchecked_into::<web_sys::HtmlInputElement>();
            slippage_bps.set(input.value());
        })
    };

    let on_cancel = {
        let on_cancel = props.on_cancel.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            on_cancel.emit(());
        })
    };

    let on_submit = {
        let strategy_id = strategy_id.clone();
        let symbol = symbol.clone();
        let start_date = start_date.clone();
        let end_date = end_date.clone();
        let initial_balance = initial_balance.clone();
        let commission_rate = commission_rate.clone();
        let slippage_bps = slippage_bps.clone();
        let loading = loading.clone();
        let error = error.clone();
        let on_success = props.on_success.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            let sid = (*strategy_id).trim().to_string();
            if sid.is_empty() {
                error.set(Some("Strategy ID is required.".to_string()));
                return;
            }

            let sym = (*symbol).trim().to_string();
            if sym.is_empty() {
                error.set(Some("Symbol is required.".to_string()));
                return;
            }

            let start_val = (*start_date).trim().to_string();
            if start_val.is_empty() {
                error.set(Some("Start date is required.".to_string()));
                return;
            }

            let end_val = (*end_date).trim().to_string();
            if end_val.is_empty() {
                error.set(Some("End date is required.".to_string()));
                return;
            }

            let balance: f64 = match initial_balance.parse() {
                Ok(v) if v > 0.0 => v,
                _ => {
                    error.set(Some("Initial balance must be a positive number.".to_string()));
                    return;
                }
            };

            let commission: Option<f64> = {
                let s = (*commission_rate).trim().to_string();
                if s.is_empty() {
                    None
                } else {
                    match s.parse::<f64>() {
                        Ok(v) => Some(v),
                        Err(_) => {
                            error.set(Some("Commission rate must be a valid number.".to_string()));
                            return;
                        }
                    }
                }
            };

            let slippage: Option<f64> = {
                let s = (*slippage_bps).trim().to_string();
                if s.is_empty() {
                    None
                } else {
                    match s.parse::<f64>() {
                        Ok(v) => Some(v),
                        Err(_) => {
                            error.set(Some("Slippage BPS must be a valid number.".to_string()));
                            return;
                        }
                    }
                }
            };

            let start_rfc = format!("{}T00:00:00Z", start_val);
            let end_rfc = format!("{}T00:00:00Z", end_val);

            error.set(None);
            loading.set(true);

            let dto = RunBacktestDto {
                strategy_id: sid,
                symbol: sym,
                start_time: start_rfc,
                end_time: end_rfc,
                initial_balance: balance,
                commission_rate: commission,
                slippage_bps: slippage,
            };

            let loading = loading.clone();
            let error = error.clone();
            let on_success = on_success.clone();

            wasm_bindgen_futures::spawn_local(async move {
                match ApiService::run_backtest(&dto).await {
                    Ok(backtest) => {
                        loading.set(false);
                        on_success.emit(backtest);
                    }
                    Err(e) => {
                        loading.set(false);
                        error.set(Some(e));
                    }
                }
            });
        })
    };

    let strategy_id_readonly = props.strategy_id.is_some();

    let input_class = css!(
        r#"
        width: 100%;
        padding: 10px 14px;
        background-color: ${bg};
        border: 1px solid ${border};
        border-radius: 8px;
        color: ${text};
        font-size: 14px;
        outline: none;
        box-sizing: border-box;
        "#,
        bg = COLORS.background,
        border = COLORS.border,
        text = COLORS.text
    );

    let label_class = css!(
        r#"
        display: block;
        font-size: 13px;
        font-weight: 600;
        color: ${muted};
        margin-bottom: 8px;
        text-transform: uppercase;
        letter-spacing: 0.05em;
        "#,
        muted = COLORS.text_muted
    );

    let field_class = css!("margin-bottom: 20px;");

    html! {
        <div class={css!(
            r#"
            position: fixed;
            inset: 0;
            background: rgba(0, 0, 0, 0.7);
            z-index: 100;
            display: flex;
            align-items: center;
            justify-content: center;
            "#
        )}>
            <div class={css!(
                r#"
                background-color: ${surface};
                border: 1px solid ${border};
                border-radius: 16px;
                padding: 36px;
                width: 520px;
                max-width: calc(100vw - 48px);
                max-height: calc(100vh - 48px);
                overflow-y: auto;
                "#,
                surface = COLORS.surface,
                border = COLORS.border
            )}>
                <h2 class={css!("font-size: 22px; font-weight: 700; margin-bottom: 8px; color: ${text};", text = COLORS.text)}>
                    { "Run Backtest" }
                </h2>
                <p class={css!("font-size: 14px; color: ${muted}; margin-bottom: 28px;", muted = COLORS.text_muted)}>
                    { "Configure and launch a historical backtest simulation." }
                </p>

                if let Some(err) = &*error {
                    <div class={css!(
                        r#"
                        color: ${danger};
                        background: rgba(239, 68, 68, 0.1);
                        border: 1px solid rgba(239, 68, 68, 0.3);
                        border-radius: 8px;
                        padding: 12px 16px;
                        font-size: 13px;
                        margin-bottom: 20px;
                        "#,
                        danger = COLORS.danger
                    )}>
                        { err }
                    </div>
                }

                <form onsubmit={on_submit}>
                    // Strategy ID
                    <div class={field_class.clone()}>
                        <label class={label_class.clone()}>{ "Strategy ID" }</label>
                        <input
                            type="text"
                            placeholder="Strategy UUID"
                            value={(*strategy_id).clone()}
                            oninput={on_strategy_id_input}
                            readonly={strategy_id_readonly}
                            class={css!(
                                r#"
                                width: 100%;
                                padding: 10px 14px;
                                background-color: ${bg};
                                border: 1px solid ${border};
                                border-radius: 8px;
                                color: ${text};
                                font-size: 14px;
                                outline: none;
                                box-sizing: border-box;
                                opacity: ${opacity};
                                "#,
                                bg = COLORS.background,
                                border = COLORS.border,
                                text = COLORS.text,
                                opacity = if strategy_id_readonly { "0.6" } else { "1" }
                            )}
                        />
                    </div>

                    // Symbol
                    <div class={field_class.clone()}>
                        <label class={label_class.clone()}>{ "Symbol" }</label>
                        <input
                            type="text"
                            placeholder="e.g. AAPL"
                            value={(*symbol).clone()}
                            oninput={on_symbol_input}
                            class={input_class.clone()}
                        />
                    </div>

                    // Start / End date
                    <div class={css!("display: grid; grid-template-columns: 1fr 1fr; gap: 16px; margin-bottom: 20px;")}>
                        <div>
                            <label class={label_class.clone()}>{ "Start Date" }</label>
                            <input
                                type="date"
                                value={(*start_date).clone()}
                                oninput={on_start_date_input}
                                class={input_class.clone()}
                            />
                        </div>
                        <div>
                            <label class={label_class.clone()}>{ "End Date" }</label>
                            <input
                                type="date"
                                value={(*end_date).clone()}
                                oninput={on_end_date_input}
                                class={input_class.clone()}
                            />
                        </div>
                    </div>

                    // Initial Balance
                    <div class={field_class.clone()}>
                        <label class={label_class.clone()}>{ "Initial Balance ($)" }</label>
                        <input
                            type="number"
                            min="1"
                            step="100"
                            value={(*initial_balance).clone()}
                            oninput={on_balance_input}
                            class={input_class.clone()}
                        />
                    </div>

                    // Commission Rate / Slippage BPS
                    <div class={css!("display: grid; grid-template-columns: 1fr 1fr; gap: 16px; margin-bottom: 28px;")}>
                        <div>
                            <label class={label_class.clone()}>{ "Commission Rate" }</label>
                            <input
                                type="number"
                                min="0"
                                step="0.0001"
                                placeholder="0.001"
                                value={(*commission_rate).clone()}
                                oninput={on_commission_input}
                                class={input_class.clone()}
                            />
                            <p class={css!("font-size: 11px; color: ${muted}; margin-top: 5px;", muted = COLORS.text_muted)}>
                                { "e.g. 0.001 = 0.1%" }
                            </p>
                        </div>
                        <div>
                            <label class={label_class.clone()}>{ "Slippage BPS" }</label>
                            <input
                                type="number"
                                min="0"
                                step="1"
                                placeholder="10"
                                value={(*slippage_bps).clone()}
                                oninput={on_slippage_input}
                                class={input_class.clone()}
                            />
                            <p class={css!("font-size: 11px; color: ${muted}; margin-top: 5px;", muted = COLORS.text_muted)}>
                                { "basis points of slippage" }
                            </p>
                        </div>
                    </div>

                    // Actions
                    <div class={css!("display: flex; gap: 12px;")}>
                        <button
                            type="button"
                            onclick={on_cancel}
                            disabled={*loading}
                            class={css!(
                                r#"
                                flex: 1;
                                padding: 11px;
                                border-radius: 8px;
                                border: 1px solid ${border};
                                background: transparent;
                                color: ${text};
                                font-size: 14px;
                                font-weight: 600;
                                cursor: pointer;
                                "#,
                                border = COLORS.border,
                                text = COLORS.text
                            )}
                        >
                            { "Cancel" }
                        </button>
                        <button
                            type="submit"
                            disabled={*loading}
                            class={css!(
                                r#"
                                flex: 1;
                                padding: 11px;
                                border-radius: 8px;
                                border: none;
                                background-color: ${success};
                                color: white;
                                font-size: 14px;
                                font-weight: 600;
                                cursor: pointer;
                                "#,
                                success = COLORS.success
                            )}
                        >
                            { if *loading { "Running…" } else { "Run Backtest" } }
                        </button>
                    </div>
                </form>
            </div>
        </div>
    }
}
