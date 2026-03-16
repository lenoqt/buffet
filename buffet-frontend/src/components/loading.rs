use crate::theme::COLORS;
use stylist::css;
use stylist::yew::styled_component;
use yew::prelude::*;

#[styled_component(Spinner)]
pub fn spinner() -> Html {
    html! {
        <div class={css!(
            r#"
            display: flex;
            align-items: center;
            justify-content: center;
            padding: 60px;
            "#
        )}>
            <div class={css!(
                r#"
                width: 40px;
                height: 40px;
                border: 3px solid ${border};
                border-top-color: ${primary};
                border-radius: 50%;
                animation: spin 0.8s linear infinite;
                @keyframes spin {
                    to { transform: rotate(360deg); }
                }
                "#,
                border = COLORS.border,
                primary = COLORS.primary
            )}></div>
        </div>
    }
}
