use crate::components::run_backtest_modal::RunBacktestModal;
use crate::components::sidebar::Sidebar;
use crate::components::strategy_form::StrategyForm;
use crate::routes::Route;
use crate::services::api::{ApiService, BacktestDto, StrategyDto};
use crate::theme::COLORS;
use stylist::css;
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(Strategies)]
pub fn strategies() -> Html {
    let strategies = use_state(|| Vec::<StrategyDto>::new());
    let error = use_state(|| None::<String>);
    let show_form = use_state(|| false);
    let run_backtest_strategy_id = use_state(|| None::<String>);
    // A counter that we increment to trigger a re-fetch
    let refresh_tick = use_state(|| 0u32);
    let navigator = use_navigator().unwrap();

    // Fetch strategies whenever refresh_tick changes
    {
        let strategies = strategies.clone();
        let error = error.clone();
        let tick = *refresh_tick;
        use_effect_with(tick, move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                match ApiService::get_strategies().await {
                    Ok(data) => strategies.set(data),
                    Err(e) => error.set(Some(e)),
                }
            });
            || ()
        });
    }

    let on_new_strategy_click = {
        let show_form = show_form.clone();
        Callback::from(move |_: MouseEvent| {
            show_form.set(true);
        })
    };

    let on_form_cancel = {
        let show_form = show_form.clone();
        Callback::from(move |_: ()| {
            show_form.set(false);
        })
    };

    let on_form_success = {
        let show_form = show_form.clone();
        let refresh_tick = refresh_tick.clone();
        Callback::from(move |_: StrategyDto| {
            show_form.set(false);
            refresh_tick.set(*refresh_tick + 1);
        })
    };

    let on_run_backtest_cancel = {
        let run_backtest_strategy_id = run_backtest_strategy_id.clone();
        Callback::from(move |_: ()| {
            run_backtest_strategy_id.set(None);
        })
    };

    let on_run_backtest_success = {
        let run_backtest_strategy_id = run_backtest_strategy_id.clone();
        let navigator = navigator.clone();
        Callback::from(move |bt: BacktestDto| {
            run_backtest_strategy_id.set(None);
            navigator.push(&Route::BacktestDetail { id: bt.id });
        })
    };

    let status_dot = |status: &str| {
        let color = if status == "active" {
            COLORS.success
        } else {
            COLORS.text_muted
        };
        css!(
            r#"
            display: inline-block;
            width: 7px;
            height: 7px;
            border-radius: 50%;
            background-color: ${color};
            margin-right: 5px;
            vertical-align: middle;
            flex-shrink: 0;
            "#,
            color = color
        )
    };

    let status_badge = |status: &str| {
        let (bg, color) = if status == "active" {
            ("rgba(16, 185, 129, 0.12)", COLORS.success)
        } else {
            ("rgba(148, 163, 184, 0.12)", COLORS.text_muted)
        };
        css!(
            r#"
            display: inline-flex;
            align-items: center;
            padding: 3px 10px;
            border-radius: 999px;
            font-size: 11px;
            font-weight: 700;
            text-transform: uppercase;
            letter-spacing: 0.05em;
            background: ${bg};
            color: ${color};
            white-space: nowrap;
            "#,
            bg = bg,
            color = color
        )
    };

    html! {
        <div class={css!(
            "display: flex; min-height: 100vh; background-color: ${bg}; color: white;",
            bg = COLORS.background
        )}>
            <Sidebar />
            <div class={css!("flex: 1; padding: 40px; overflow-y: auto;")}>

                // Header row
                <div class={css!("display: flex; justify-content: space-between; align-items: center; margin-bottom: 32px;")}>
                    <h1 class={css!("font-size: 32px; font-weight: 700;")}>{ "Strategies" }</h1>
                    <button
                        onclick={on_new_strategy_click}
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
                        { "New Strategy" }
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

                // Strategy grid or empty state
                {
                    if strategies.is_empty() {
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
                                <div class={css!("font-size: 40px; margin-bottom: 16px;")}>{ "📊" }</div>
                                <div>{ "No strategies found. Create one to get started." }</div>
                            </div>
                        }
                    } else {
                        let navigator = navigator.clone();
                        let run_backtest_strategy_id_outer = run_backtest_strategy_id.clone();
                        html! {
                            <div class={css!("display: grid; grid-template-columns: repeat(auto-fill, minmax(320px, 1fr)); gap: 24px;")}>
                                {
                                    strategies.iter().map(|s| {
                                        let sid = s.id.clone();
                                        let nav = navigator.clone();
                                        let rbsid = run_backtest_strategy_id_outer.clone();

                                        let on_configure = {
                                            let sid = sid.clone();
                                            let nav = nav.clone();
                                            Callback::from(move |_: MouseEvent| {
                                                nav.push(&Route::StrategyDetail { id: sid.clone() });
                                            })
                                        };

                                        let on_run = {
                                            let sid = sid.clone();
                                            let rbsid = rbsid.clone();
                                            Callback::from(move |_: MouseEvent| {
                                                rbsid.set(Some(sid.clone()));
                                            })
                                        };

                                        // Parse symbols JSON array for display chips
                                        let symbols_display: Vec<String> =
                                            serde_json::from_str::<Vec<String>>(&s.symbols)
                                                .unwrap_or_else(|_| {
                                                    s.symbols
                                                        .split(',')
                                                        .map(|x| x.trim().to_string())
                                                        .filter(|x| !x.is_empty())
                                                        .collect()
                                                });

                                        let status = s.status.clone();

                                        html! {
                                            <div class={css!(
                                                r#"
                                                background-color: ${surface};
                                                border: 1px solid ${border};
                                                border-radius: 16px;
                                                padding: 24px;
                                                display: flex;
                                                flex-direction: column;
                                                "#,
                                                surface = COLORS.surface,
                                                border = COLORS.border
                                            )}>
                                                // Top row: name + status badge
                                                <div class={css!("display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 8px;")}>
                                                    <h3 class={css!(
                                                        "font-size: 17px; font-weight: 700; color: ${text}; margin: 0; flex: 1; min-width: 0;",
                                                        text = COLORS.text
                                                    )}>
                                                        { &s.name }
                                                    </h3>
                                                    <span class={status_badge(&status)}>
                                                        <span class={status_dot(&status)}></span>
                                                        { &status }
                                                    </span>
                                                </div>

                                                // Strategy type label
                                                <div class={css!(
                                                    "font-size: 12px; color: ${primary}; font-weight: 600; text-transform: uppercase; letter-spacing: 0.05em; margin-bottom: 14px;",
                                                    primary = COLORS.primary
                                                )}>
                                                    { &s.strategy_type }
                                                </div>

                                                // Symbol chips
                                                <div class={css!("display: flex; flex-wrap: wrap; gap: 6px; margin-bottom: 16px;")}>
                                                    {
                                                        symbols_display.iter().map(|sym| html! {
                                                            <span class={css!(
                                                                r#"
                                                                padding: 2px 10px;
                                                                border-radius: 999px;
                                                                border: 1px solid ${border};
                                                                font-size: 11px;
                                                                font-weight: 600;
                                                                color: ${muted};
                                                                "#,
                                                                border = COLORS.border,
                                                                muted = COLORS.text_muted
                                                            )}>
                                                                { sym }
                                                            </span>
                                                        }).collect::<Html>()
                                                    }
                                                </div>

                                                // Created at
                                                <div class={css!(
                                                    "font-size: 12px; color: ${muted}; margin-bottom: 20px; flex: 1;",
                                                    muted = COLORS.text_muted
                                                )}>
                                                    { format!("Created: {}", &s.created_at) }
                                                </div>

                                                // Action buttons
                                                <div class={css!("display: flex; gap: 10px;")}>
                                                    <button
                                                        onclick={on_configure}
                                                        class={css!(
                                                            r#"
                                                            flex: 1;
                                                            padding: 9px;
                                                            border-radius: 8px;
                                                            border: 1px solid ${border};
                                                            background: transparent;
                                                            color: ${text};
                                                            font-size: 13px;
                                                            font-weight: 600;
                                                            cursor: pointer;
                                                            transition: background 0.15s;
                                                            "#,
                                                            border = COLORS.border,
                                                            text = COLORS.text
                                                        )}
                                                    >
                                                        { "Configure" }
                                                    </button>
                                                    <button
                                                        onclick={on_run}
                                                        class={css!(
                                                            r#"
                                                            flex: 1;
                                                            padding: 9px;
                                                            border-radius: 8px;
                                                            border: none;
                                                            background: ${success};
                                                            color: white;
                                                            font-size: 13px;
                                                            font-weight: 600;
                                                            cursor: pointer;
                                                            transition: opacity 0.15s;
                                                            "#,
                                                            success = COLORS.success
                                                        )}
                                                    >
                                                        { "Run" }
                                                    </button>
                                                </div>
                                            </div>
                                        }
                                    }).collect::<Html>()
                                }
                            </div>
                        }
                    }
                }
            </div>

            // New Strategy modal
            if *show_form {
                <StrategyForm
                    on_success={on_form_success}
                    on_cancel={on_form_cancel}
                />
            }

            // Run Backtest modal (pre-filled strategy id)
            if (*run_backtest_strategy_id).is_some() {
                <RunBacktestModal
                    strategy_id={(*run_backtest_strategy_id).clone()}
                    on_success={on_run_backtest_success}
                    on_cancel={on_run_backtest_cancel}
                />
            }
        </div>
    }
}
