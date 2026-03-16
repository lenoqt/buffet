use crate::components::loading::Spinner;
use crate::components::sidebar::Sidebar;
use crate::routes::Route;
use crate::services::api::{ApiService, StrategyDto};
use crate::theme::COLORS;
use stylist::css;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq)]
pub struct StrategyDetailProps {
    pub id: String,
}

#[function_component(StrategyDetail)]
pub fn strategy_detail(props: &StrategyDetailProps) -> Html {
    let strategy = use_state(|| None::<StrategyDto>);
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);
    let action_error = use_state(|| None::<String>);
    let action_loading = use_state(|| false);
    let navigator = use_navigator().unwrap();

    {
        let strategy = strategy.clone();
        let loading = loading.clone();
        let error = error.clone();
        let id = props.id.clone();
        use_effect_with(id.clone(), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                match ApiService::get_strategy(&id).await {
                    Ok(data) => {
                        strategy.set(Some(data));
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

    let on_activate = {
        let strategy = strategy.clone();
        let action_loading = action_loading.clone();
        let action_error = action_error.clone();
        let id = props.id.clone();
        Callback::from(move |_: MouseEvent| {
            let strategy = strategy.clone();
            let action_loading = action_loading.clone();
            let action_error = action_error.clone();
            let id = id.clone();
            action_loading.set(true);
            action_error.set(None);
            wasm_bindgen_futures::spawn_local(async move {
                match ApiService::activate_strategy(&id).await {
                    Ok(updated) => {
                        strategy.set(Some(updated));
                        action_loading.set(false);
                    }
                    Err(e) => {
                        action_error.set(Some(e));
                        action_loading.set(false);
                    }
                }
            });
        })
    };

    let on_deactivate = {
        let strategy = strategy.clone();
        let action_loading = action_loading.clone();
        let action_error = action_error.clone();
        let id = props.id.clone();
        Callback::from(move |_: MouseEvent| {
            let strategy = strategy.clone();
            let action_loading = action_loading.clone();
            let action_error = action_error.clone();
            let id = id.clone();
            action_loading.set(true);
            action_error.set(None);
            wasm_bindgen_futures::spawn_local(async move {
                match ApiService::deactivate_strategy(&id).await {
                    Ok(updated) => {
                        strategy.set(Some(updated));
                        action_loading.set(false);
                    }
                    Err(e) => {
                        action_error.set(Some(e));
                        action_loading.set(false);
                    }
                }
            });
        })
    };

    let on_back = {
        let navigator = navigator.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            navigator.push(&Route::Strategies);
        })
    };

    let badge_css = |status: &str| {
        let (bg, color) = if status == "active" {
            ("rgba(16, 185, 129, 0.15)", COLORS.success)
        } else {
            ("rgba(148, 163, 184, 0.15)", COLORS.text_muted)
        };
        css!(
            r#"
            display: inline-flex;
            align-items: center;
            gap: 6px;
            padding: 4px 12px;
            border-radius: 999px;
            font-size: 12px;
            font-weight: 600;
            text-transform: uppercase;
            letter-spacing: 0.05em;
            background: ${bg};
            color: ${color};
            "#,
            bg = bg,
            color = color
        )
    };

    let dot_css = |status: &str| {
        let color = if status == "active" {
            COLORS.success
        } else {
            COLORS.text_muted
        };
        css!(
            r#"
            width: 7px;
            height: 7px;
            border-radius: 50%;
            background-color: ${color};
            "#,
            color = color
        )
    };

    let section_label = css!(
        r#"
        font-size: 11px;
        font-weight: 700;
        text-transform: uppercase;
        letter-spacing: 0.08em;
        color: ${muted};
        margin-bottom: 10px;
        "#,
        muted = COLORS.text_muted
    );

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
                    { "← Back to Strategies" }
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
                                { format!("Failed to load strategy: {}", err) }
                            </div>
                        }
                    } else if let Some(s) = &*strategy {
                        let is_active = s.status == "active";
                        html! {
                            <>
                                // Header
                                <div class={css!("display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 32px;")}>
                                    <div>
                                        <h1 class={css!("font-size: 32px; font-weight: 700; margin-bottom: 12px;")}>
                                            { &s.name }
                                        </h1>
                                        <div class={css!("display: flex; align-items: center; gap: 12px;")}>
                                            <span class={css!(
                                                "font-size: 13px; font-weight: 600; color: ${primary}; text-transform: uppercase;",
                                                primary = COLORS.primary
                                            )}>
                                                { &s.strategy_type }
                                            </span>
                                            <span class={badge_css(&s.status)}>
                                                <span class={dot_css(&s.status)}></span>
                                                { &s.status }
                                            </span>
                                        </div>
                                    </div>

                                    // Action buttons
                                    <div class={css!("display: flex; gap: 12px; align-items: center;")}>
                                        if let Some(aerr) = &*action_error {
                                            <span class={css!("font-size: 13px; color: ${danger};", danger = COLORS.danger)}>
                                                { aerr }
                                            </span>
                                        }
                                        if is_active {
                                            <button
                                                onclick={on_deactivate}
                                                disabled={*action_loading}
                                                class={css!(
                                                    r#"
                                                    padding: 10px 20px;
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
                                                { if *action_loading { "Updating…" } else { "Deactivate" } }
                                            </button>
                                        } else {
                                            <button
                                                onclick={on_activate}
                                                disabled={*action_loading}
                                                class={css!(
                                                    r#"
                                                    padding: 10px 20px;
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
                                                { if *action_loading { "Updating…" } else { "Activate" } }
                                            </button>
                                        }
                                    </div>
                                </div>

                                // Overview card
                                <div class={card_css.clone()}>
                                    <div class={section_label.clone()}>{ "Overview" }</div>
                                    <div class={css!("display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 24px;")}>
                                        <div>
                                            <div class={css!("font-size: 12px; color: ${muted}; margin-bottom: 4px;", muted = COLORS.text_muted)}>{ "Strategy ID" }</div>
                                            <div class={css!("font-size: 13px; font-family: monospace; word-break: break-all;")}>{ &s.id }</div>
                                        </div>
                                        <div>
                                            <div class={css!("font-size: 12px; color: ${muted}; margin-bottom: 4px;", muted = COLORS.text_muted)}>{ "Created At" }</div>
                                            <div class={css!("font-size: 13px;")}>{ &s.created_at }</div>
                                        </div>
                                        <div>
                                            <div class={css!("font-size: 12px; color: ${muted}; margin-bottom: 4px;", muted = COLORS.text_muted)}>{ "Status" }</div>
                                            <div class={css!("font-size: 13px;")}>{ &s.status }</div>
                                        </div>
                                    </div>
                                </div>

                                // Symbols card
                                <div class={card_css.clone()}>
                                    <div class={section_label.clone()}>{ "Symbols" }</div>
                                    <div class={css!("display: flex; flex-wrap: wrap; gap: 10px;")}>
                                        {
                                            // symbols is stored as a JSON array string, e.g. ["AAPL","MSFT"]
                                            // parse it; fall back to displaying the raw string as a single chip
                                            let raw = &s.symbols;
                                            let chips: Vec<String> = serde_json::from_str::<Vec<String>>(raw)
                                                .unwrap_or_else(|_| {
                                                    raw.split(',')
                                                        .map(|x| x.trim().to_string())
                                                        .filter(|x| !x.is_empty())
                                                        .collect()
                                                });
                                            chips.iter().map(|sym| html! {
                                                <span class={css!(
                                                    r#"
                                                    padding: 4px 14px;
                                                    border-radius: 999px;
                                                    border: 1px solid ${border};
                                                    font-size: 13px;
                                                    font-weight: 600;
                                                    color: ${primary};
                                                    "#,
                                                    border = COLORS.border,
                                                    primary = COLORS.primary
                                                )}>
                                                    { sym }
                                                </span>
                                            }).collect::<Html>()
                                        }
                                    </div>
                                </div>

                                // Parameters card
                                <div class={card_css.clone()}>
                                    <div class={section_label.clone()}>{ "Parameters" }</div>
                                    <pre class={css!(
                                        r#"
                                        margin: 0;
                                        font-family: 'JetBrains Mono', 'Fira Code', monospace;
                                        font-size: 13px;
                                        color: ${text};
                                        background-color: ${bg};
                                        border-radius: 8px;
                                        padding: 16px;
                                        overflow-x: auto;
                                        white-space: pre-wrap;
                                        word-break: break-word;
                                        "#,
                                        text = COLORS.text,
                                        bg = COLORS.background
                                    )}>
                                        {
                                            // Pretty-print JSON if possible
                                            serde_json::from_str::<serde_json::Value>(&s.parameters)
                                                .map(|v| serde_json::to_string_pretty(&v).unwrap_or_else(|_| s.parameters.clone()))
                                                .unwrap_or_else(|_| s.parameters.clone())
                                        }
                                    </pre>
                                </div>
                            </>
                        }
                    } else {
                        html! {
                            <div class={css!("color: ${muted};", muted = COLORS.text_muted)}>
                                { "Strategy not found." }
                            </div>
                        }
                    }
                }
            </div>
        </div>
    }
}
