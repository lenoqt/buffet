use crate::components::sidebar::Sidebar;
use crate::services::api::{ApiService, StrategyDto};
use crate::theme::COLORS;
use stylist::css;
use yew::prelude::*;

#[function_component(Strategies)]
pub fn strategies() -> Html {
    let strategies = use_state(|| Vec::<StrategyDto>::new());
    let error = use_state(|| None::<String>);

    {
        let strategies = strategies.clone();
        let error = error.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                match ApiService::get_strategies().await {
                    Ok(data) => strategies.set(data),
                    Err(e) => error.set(Some(e)),
                }
            });
            || ()
        });
    }

    html! {
        <div class={css!("display: flex; min-height: 100vh; background-color: ${bg}; color: white;", bg = COLORS.background)}>
            <Sidebar />
            <div class={css!("flex: 1; padding: 40px; overflow-y: auto;")}>
                <div class={css!("display: flex; justify-content: space-between; align-items: center; margin-bottom: 32px;")}>
                    <h1 class={css!("font-size: 32px; font-weight: 700;")}>{ "Strategies" }</h1>
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
                        { "New Strategy" }
                    </button>
                </div>

                {
                    if let Some(err) = &*error {
                        html! { <div class={css!("color: ${danger}; padding: 16px; background: rgba(239, 68, 68, 0.1); border-radius: 8px; margin-bottom: 24px;", danger = COLORS.danger)}>{ err }</div> }
                    } else if strategies.is_empty() {
                        html! { <div class={css!("color: ${text_muted};", text_muted = COLORS.text_muted)}>{ "No strategies found." }</div> }
                    } else {
                        html! {
                            <div class={css!("display: grid; grid-template-columns: repeat(auto-fill, minmax(300px, 1fr)); gap: 24px;")}>
                                {
                                    strategies.iter().map(|s| html! {
                                        <div class={css!(
                                            r#"
                                            background-color: ${surface};
                                            border: 1px solid ${border};
                                            border-radius: 16px;
                                            padding: 24px;
                                            "#,
                                            surface = COLORS.surface,
                                            border = COLORS.border
                                        )}>
                                            <h3 class={css!("margin-bottom: 8px;")}>{ &s.name }</h3>
                                            <div class={css!("font-size: 12px; color: ${primary}; font-weight: 600; text-transform: uppercase; margin-bottom: 16px;", primary = COLORS.primary)}>
                                                { &s.strategy_type }
                                            </div>
                                            <div class={css!("font-size: 13px; color: ${text_muted}; margin-bottom: 24px;", text_muted = COLORS.text_muted)}>
                                                { format!("Created: {}", &s.created_at) }
                                            </div>
                                            <div class={css!("display: flex; gap: 12px;")}>
                                                <button class={css!("flex: 1; padding: 8px; border-radius: 6px; border: 1px solid ${border}; background: transparent; color: white; cursor: pointer;", border = COLORS.border)}>{ "Configure" }</button>
                                                <button class={css!("flex: 1; padding: 8px; border-radius: 6px; border: none; background: ${success}; color: white; cursor: pointer;", success = COLORS.success)}>{ "Run" }</button>
                                            </div>
                                        </div>
                                    }).collect::<Html>()
                                }
                            </div>
                        }
                    }
                }
            </div>
        </div>
    }
}
