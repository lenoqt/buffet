use crate::components::sidebar::Sidebar;
use crate::services::api::{ApiService, OrderDto, PositionDto};
use crate::theme::COLORS;
use stylist::css;
use stylist::yew::styled_component;
use yew::prelude::*;

#[styled_component(Dashboard)]
pub fn dashboard() -> Html {
    let orders = use_state(|| Vec::<OrderDto>::new());
    let positions = use_state(|| Vec::<PositionDto>::new());
    let error = use_state(|| None::<String>);

    {
        let orders = orders.clone();
        let positions = positions.clone();
        let error = error.clone();

        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                // Fetch Orders
                match ApiService::get_orders().await {
                    Ok(data) => orders.set(data),
                    Err(e) => error.set(Some(format!("Orders: {}", e))),
                }

                // Fetch Positions
                match ApiService::get_positions().await {
                    Ok(data) => positions.set(data),
                    Err(e) => error.set(Some(format!("Positions: {}", e))),
                }
            });
            || ()
        });
    }

    // Calculate dynamic stats
    let active_trades = positions.iter().filter(|p| p.status == "open").count();
    let total_pnl: f64 = positions.iter().filter_map(|p| p.unrealized_pnl).sum();
    let last_trades: Vec<OrderDto> = orders.iter().take(5).cloned().collect();

    html! {
        <div class={css!("display: flex; min-height: 100vh; background-color: ${bg}; color: white;", bg = COLORS.background)}>
            <Sidebar />

            // Main Content
            <main class={css!("flex: 1; padding: 40px; overflow-y: auto;")}>
                {
                    if let Some(err) = &*error {
                        html! {
                            <div class={css!("color: ${danger}; padding: 16px; background: rgba(239, 68, 68, 0.1); border-radius: 8px; margin-bottom: 24px;", danger = COLORS.danger)}>
                                { format!("API Error: {}", err) }
                            </div>
                        }
                    } else {
                        html! {}
                    }
                }

                <div class={css!("display: flex; justify-content: space-between; align-items: flex-end; margin-bottom: 32px;")}>
                    <div>
                        <h1 class={css!("font-size: 32px; font-weight: 700; margin-bottom: 8px;")}>{ "Dashboard" }</h1>
                        <p class={css!("color: ${text_muted}; font-size: 14px;", text_muted = COLORS.text_muted)}>
                            { "Welcome back. Your strategies are running smoothly." }
                        </p>
                    </div>
                    <button class={css!(
                        r#"
                        background-color: ${primary};
                        color: white;
                        border: none;
                        padding: 10px 20px;
                        border-radius: 8px;
                        font-weight: 600;
                        cursor: pointer;
                        transition: background-color 0.2s;
                        &:hover {
                            background-color: ${hover};
                        }
                        "#,
                        primary = COLORS.primary,
                        hover = COLORS.primary_hover
                    )}>
                        { "+ Create Strategy" }
                    </button>
                </div>

                // Stats Grid
                <div class={css!("display: grid; grid-template-columns: repeat(4, 1fr); gap: 24px; margin-bottom: 40px;")}>
                    <StatCard label="Total Equity" value="$10,000.00" change="0%" positive=true />
                    <StatCard label="Active Trades" value={active_trades.to_string()} change="0" positive=true />
                    <StatCard label="Profit (unrealized)" value={format!("${:.2}", total_pnl)} change={if total_pnl >= 0.0 { "+0%" } else { "-0%" }} positive={total_pnl >= 0.0} />
                    <StatCard label="Order Count" value={orders.len().to_string()} change="" positive=true />
                </div>

                // Charts Row
                <div class={css!("display: grid; grid-template-columns: 2fr 1fr; gap: 24px;")}>
                    <div class={css!(
                        r#"
                        background-color: ${surface};
                        border: 1px solid ${border};
                        border-radius: 16px;
                        padding: 24px;
                        height: 400px;
                        "#,
                        surface = COLORS.surface,
                        border = COLORS.border
                    )}>
                        <h3 class={css!("margin-bottom: 24px;")}>{ "Performance Overview" }</h3>
                        <div style="height: 300px; display: flex; align-items: center; justify-content: center; color: #4b4b55; border: 1px dashed #2d2d35; border-radius: 8px;">
                            { "Real-time chart visualization coming soon" }
                        </div>
                    </div>

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
                        <h3 class={css!("margin-bottom: 24px;")}>{ "Recent Orders" }</h3>
                        <div style="display: flex; flex-direction: column; gap: 16px;">
                            {
                                if last_trades.is_empty() {
                                    html! { <div style="color: #4b4b55; font-size: 14px;">{ "No recent orders to show." }</div> }
                                } else {
                                    last_trades.iter().map(|o| html! {
                                        <TradeItem
                                            symbol={o.symbol.clone()}
                                            type_={o.side.clone()}
                                            price={format!("${:.2}", o.price.unwrap_or(0.0))}
                                            status={o.status.clone()}
                                        />
                                    }).collect::<Html>()
                                }
                            }
                        </div>
                    </div>
                </div>
            </main>
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct StatCardProps {
    label: &'static str,
    value: String,
    change: &'static str,
    positive: bool,
}

#[styled_component(StatCard)]
fn stat_card(props: &StatCardProps) -> Html {
    html! {
        <div class={css!(
            r#"
            background-color: ${surface};
            border: 1px solid ${border};
            border-radius: 16px;
            padding: 20px;
            "#,
            surface = COLORS.surface,
            border = COLORS.border
        )}>
            <div class={css!("color: ${text_muted}; font-size: 12px; font-weight: 600; text-transform: uppercase; margin-bottom: 12px;", text_muted = COLORS.text_muted)}>
                { props.label }
            </div>
            <div class={css!("font-size: 24px; font-weight: 700; margin-bottom: 8px;")}>
                { props.value.clone() }
            </div>
            <div class={css!(
                r#"
                font-size: 13px;
                font-weight: 600;
                color: ${color};
                "#,
                color = if props.positive { COLORS.success } else { COLORS.danger }
            )}>
                { props.change }
                if !props.change.is_empty() {
                    <span style="color: #4b4b55; font-weight: 400; margin-left: 4px;">{ "vs last period" }</span>
                }
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct TradeItemProps {
    symbol: String,
    type_: String,
    price: String,
    status: String,
}

#[styled_component(TradeItem)]
fn trade_item(props: &TradeItemProps) -> Html {
    html! {
        <div class={css!(
            r#"
            display: flex;
            align-items: center;
            justify-content: space-between;
            padding-bottom: 12px;
            border-bottom: 1px solid ${border};
            &:last-child {
                border-bottom: none;
                padding-bottom: 0;
            }
            "#,
            border = COLORS.border
        )}>
            <div>
                <div class={css!("font-size: 14px; font-weight: 600; margin-bottom: 2px;")}>{ props.symbol.clone() }</div>
                <div class={css!(
                    r#"
                    font-size: 12px;
                    font-weight: 500;
                    color: ${color};
                    "#,
                    color = if props.type_ == "buy" || props.type_ == "Buy" { COLORS.success } else { COLORS.danger }
                )}>{ props.type_.clone() }</div>
            </div>
            <div style="text-align: right;">
                <div class={css!("font-size: 14px; font-weight: 500; margin-bottom: 2px;")}>{ props.price.clone() }</div>
                <div class={css!("font-size: 11px; color: ${text_muted};", text_muted = COLORS.text_muted)}>{ props.status.clone() }</div>
            </div>
        </div>
    }
}
