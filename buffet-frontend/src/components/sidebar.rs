use crate::routes::Route;
use crate::theme::COLORS;
use stylist::css;
use stylist::yew::styled_component;
use yew::prelude::*;
use yew_router::prelude::*;

#[styled_component(Sidebar)]
pub fn sidebar() -> Html {
    html! {
        <aside class={css!(
            r#"
            width: 260px;
            background-color: ${surface};
            border-right: 1px solid ${border};
            height: 100vh;
            display: flex;
            flex-direction: column;
            padding: 24px 16px;
            position: sticky;
            top: 0;
            "#,
            surface = COLORS.surface,
            border = COLORS.border
        )}>
            <Link<Route> to={Route::Dashboard} classes={css!("text-decoration: none;")}>
                <div class={css!(
                    r#"
                    font-size: 24px;
                    font-weight: 800;
                    color: ${primary};
                    margin-bottom: 48px;
                    display: flex;
                    align-items: center;
                    gap: 12px;
                    font-family: 'Outfit', sans-serif;
                    "#,
                    primary = COLORS.primary
                )}>
                    <div class={css!(
                        r#"
                        width: 32px;
                        height: 32px;
                        background: linear-gradient(135deg, ${p} 0%, ${s} 100%);
                        border-radius: 8px;
                        "#,
                        p = COLORS.primary,
                        s = COLORS.secondary
                    )}></div>
                    { "BUFFET" }
                </div>
            </Link<Route>>

            <nav class={css!("display: flex; flex-direction: column; gap: 8px;")}>
                <SidebarItem route={Route::Dashboard} label="Dashboard" />
                <SidebarItem route={Route::Strategies} label="Strategies" />
                <SidebarItem route={Route::Backtests} label="Backtests" />

                <div style="margin-top: 24px; margin-bottom: 8px; font-size: 11px; font-weight: 600; color: #4b4b55; text-transform: uppercase; letter-spacing: 0.05em; padding-left: 12px;">{ "Trading" }</div>
                <SidebarItem route={Route::Dashboard} label="Orders" />
                <SidebarItem route={Route::Dashboard} label="Positions" />
            </nav>

            <div style="flex: 1"></div>

            <nav class={css!("display: flex; flex-direction: column; gap: 8px;")}>
                <SidebarItem route={Route::Settings} label="Settings" />
                <div class={css!(
                    r#"
                    display: flex;
                    align-items: center;
                    gap: 12px;
                    padding: 10px 12px;
                    border-radius: 8px;
                    cursor: pointer;
                    color: ${muted};
                    &:hover {
                        background-color: ${hover};
                        color: ${text};
                    }
                    "#,
                    muted = COLORS.text_muted,
                    hover = COLORS.surface_light,
                    text = COLORS.text
                )}>
                    <span class={css!("font-size: 14px; font-weight: 500;")}>{ "Sign Out" }</span>
                </div>
            </nav>
        </aside>
    }
}

#[derive(Properties, PartialEq)]
struct SidebarItemProps {
    route: Route,
    label: &'static str,
}

#[styled_component(SidebarItem)]
fn sidebar_item(props: &SidebarItemProps) -> Html {
    let current_route = use_route::<Route>();
    let active = current_route == Some(props.route.clone());

    html! {
        <Link<Route> to={props.route.clone()} classes={css!("text-decoration: none;")}>
            <div class={css!(
                r#"
                display: flex;
                align-items: center;
                gap: 12px;
                padding: 10px 12px;
                border-radius: 8px;
                cursor: pointer;
                transition: all 0.2s;
                background-color: ${bg};
                color: ${color};
                &:hover {
                    background-color: ${hover};
                    color: ${text};
                }
                "#,
                bg = if active { COLORS.surface_light } else { "transparent" },
                color = if active { COLORS.text } else { COLORS.text_muted },
                hover = COLORS.surface_light,
                text = COLORS.text
            )}>
                <span class={css!("font-size: 14px; font-weight: 500;")}>{ props.label }</span>
            </div>
        </Link<Route>>
    }
}
