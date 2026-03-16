
use crate::services::api::{ApiService, CreateStrategyDto, StrategyDto};
use crate::theme::COLORS;
use stylist::css;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct StrategyFormProps {
    pub on_success: Callback<StrategyDto>,
    pub on_cancel: Callback<()>,
}

#[function_component(StrategyForm)]
pub fn strategy_form(props: &StrategyFormProps) -> Html {
    let name = use_state(|| String::new());
    let strategy_type = use_state(|| "Classical".to_string());
    let fast_period = use_state(|| "10".to_string());
    let slow_period = use_state(|| "30".to_string());
    let symbols = use_state(|| String::new());
    let loading = use_state(|| false);
    let error = use_state(|| None::<String>);

    let on_name_input = {
        let name = name.clone();
        Callback::from(move |e: InputEvent| {
            let input = e.target_unchecked_into::<web_sys::HtmlInputElement>();
            name.set(input.value());
        })
    };

    let on_type_change = {
        let strategy_type = strategy_type.clone();
        Callback::from(move |e: Event| {
            let select = e.target_unchecked_into::<web_sys::HtmlSelectElement>();
            strategy_type.set(select.value());
        })
    };

    let on_fast_input = {
        let fast_period = fast_period.clone();
        Callback::from(move |e: InputEvent| {
            let input = e.target_unchecked_into::<web_sys::HtmlInputElement>();
            fast_period.set(input.value());
        })
    };

    let on_slow_input = {
        let slow_period = slow_period.clone();
        Callback::from(move |e: InputEvent| {
            let input = e.target_unchecked_into::<web_sys::HtmlInputElement>();
            slow_period.set(input.value());
        })
    };

    let on_symbols_input = {
        let symbols = symbols.clone();
        Callback::from(move |e: InputEvent| {
            let input = e.target_unchecked_into::<web_sys::HtmlInputElement>();
            symbols.set(input.value());
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
        let name = name.clone();
        let strategy_type = strategy_type.clone();
        let fast_period = fast_period.clone();
        let slow_period = slow_period.clone();
        let symbols = symbols.clone();
        let loading = loading.clone();
        let error = error.clone();
        let on_success = props.on_success.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            let name_val = (*name).trim().to_string();
            if name_val.is_empty() {
                error.set(Some("Strategy name is required.".to_string()));
                return;
            }

            let symbols_val = (*symbols).trim().to_string();
            if symbols_val.is_empty() {
                error.set(Some("At least one symbol is required.".to_string()));
                return;
            }

            let symbol_list: Vec<String> = symbols_val
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            if symbol_list.is_empty() {
                error.set(Some("At least one valid symbol is required.".to_string()));
                return;
            }

            let parameters = if *strategy_type == "Classical" {
                let fast: u32 = fast_period.parse().unwrap_or(10);
                let slow: u32 = slow_period.parse().unwrap_or(30);
                if fast >= slow {
                    error.set(Some(
                        "Fast period must be less than slow period.".to_string(),
                    ));
                    return;
                }
                format!(
                    "{{\"fast_period\":{}, \"slow_period\":{}}}",
                    fast, slow
                )
            } else {
                "{}".to_string()
            };

            error.set(None);
            loading.set(true);

            let dto = CreateStrategyDto {
                name: name_val,
                strategy_type: (*strategy_type).clone(),
                parameters,
                symbols: symbol_list,
            };

            let loading = loading.clone();
            let error = error.clone();
            let on_success = on_success.clone();

            wasm_bindgen_futures::spawn_local(async move {
                match ApiService::create_strategy(&dto).await {
                    Ok(created) => {
                        loading.set(false);
                        on_success.emit(created);
                    }
                    Err(e) => {
                        loading.set(false);
                        error.set(Some(e));
                    }
                }
            });
        })
    };

    let is_classical = *strategy_type == "Classical";

    html! {
        // Modal overlay
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
            // Modal card
            <div class={css!(
                r#"
                background-color: ${surface};
                border: 1px solid ${border};
                border-radius: 16px;
                padding: 36px;
                width: 480px;
                max-width: calc(100vw - 48px);
                max-height: calc(100vh - 48px);
                overflow-y: auto;
                "#,
                surface = COLORS.surface,
                border = COLORS.border
            )}>
                <h2 class={css!("font-size: 22px; font-weight: 700; margin-bottom: 8px; color: ${text};", text = COLORS.text)}>
                    { "New Strategy" }
                </h2>
                <p class={css!("font-size: 14px; color: ${muted}; margin-bottom: 28px;", muted = COLORS.text_muted)}>
                    { "Configure and deploy a new trading strategy." }
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
                    // Name
                    <div class={css!("margin-bottom: 20px;")}>
                        <label class={css!("display: block; font-size: 13px; font-weight: 600; color: ${muted}; margin-bottom: 8px; text-transform: uppercase; letter-spacing: 0.05em;", muted = COLORS.text_muted)}>
                            { "Strategy Name" }
                        </label>
                        <input
                            type="text"
                            placeholder="e.g. My MA Crossover"
                            value={(*name).clone()}
                            oninput={on_name_input}
                            required={true}
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
                                "#,
                                bg = COLORS.background,
                                border = COLORS.border,
                                text = COLORS.text
                            )}
                        />
                    </div>

                    // Strategy Type
                    <div class={css!("margin-bottom: 20px;")}>
                        <label class={css!("display: block; font-size: 13px; font-weight: 600; color: ${muted}; margin-bottom: 8px; text-transform: uppercase; letter-spacing: 0.05em;", muted = COLORS.text_muted)}>
                            { "Strategy Type" }
                        </label>
                        <select
                            onchange={on_type_change}
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
                                cursor: pointer;
                                "#,
                                bg = COLORS.background,
                                border = COLORS.border,
                                text = COLORS.text
                            )}
                        >
                            <option value="Classical" selected={*strategy_type == "Classical"}>{ "Classical" }</option>
                            <option value="Statistical" selected={*strategy_type == "Statistical"}>{ "Statistical" }</option>
                            <option value="ML-Based" selected={*strategy_type == "ML-Based"}>{ "ML-Based" }</option>
                        </select>
                    </div>

                    // Fast / Slow period — only for Classical
                    if is_classical {
                        <div class={css!("display: grid; grid-template-columns: 1fr 1fr; gap: 16px; margin-bottom: 20px;")}>
                            <div>
                                <label class={css!("display: block; font-size: 13px; font-weight: 600; color: ${muted}; margin-bottom: 8px; text-transform: uppercase; letter-spacing: 0.05em;", muted = COLORS.text_muted)}>
                                    { "Fast Period" }
                                </label>
                                <input
                                    type="number"
                                    min="1"
                                    value={(*fast_period).clone()}
                                    oninput={on_fast_input}
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
                                        "#,
                                        bg = COLORS.background,
                                        border = COLORS.border,
                                        text = COLORS.text
                                    )}
                                />
                            </div>
                            <div>
                                <label class={css!("display: block; font-size: 13px; font-weight: 600; color: ${muted}; margin-bottom: 8px; text-transform: uppercase; letter-spacing: 0.05em;", muted = COLORS.text_muted)}>
                                    { "Slow Period" }
                                </label>
                                <input
                                    type="number"
                                    min="1"
                                    value={(*slow_period).clone()}
                                    oninput={on_slow_input}
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
                                        "#,
                                        bg = COLORS.background,
                                        border = COLORS.border,
                                        text = COLORS.text
                                    )}
                                />
                            </div>
                        </div>
                    }

                    // Symbols
                    <div class={css!("margin-bottom: 28px;")}>
                        <label class={css!("display: block; font-size: 13px; font-weight: 600; color: ${muted}; margin-bottom: 8px; text-transform: uppercase; letter-spacing: 0.05em;", muted = COLORS.text_muted)}>
                            { "Symbols" }
                        </label>
                        <input
                            type="text"
                            placeholder="e.g. AAPL, MSFT, TSLA"
                            value={(*symbols).clone()}
                            oninput={on_symbols_input}
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
                                "#,
                                bg = COLORS.background,
                                border = COLORS.border,
                                text = COLORS.text
                            )}
                        />
                        <p class={css!("font-size: 12px; color: ${muted}; margin-top: 6px;", muted = COLORS.text_muted)}>
                            { "Comma-separated list of ticker symbols" }
                        </p>
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
                                background-color: ${primary};
                                color: white;
                                font-size: 14px;
                                font-weight: 600;
                                cursor: pointer;
                                "#,
                                primary = COLORS.primary
                            )}
                        >
                            { if *loading { "Creating…" } else { "Create Strategy" } }
                        </button>
                    </div>
                </form>
            </div>
        </div>
    }
}
