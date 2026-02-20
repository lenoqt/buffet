use stylist::css;
use stylist::yew::Global;
use yew::prelude::*;

pub const COLORS: Colors = Colors {
    background: "#0c0c0e",
    surface: "#16161a",
    surface_light: "#212127",
    primary: "#8b5cf6", // Violet 500
    primary_hover: "#7c3aed",
    secondary: "#3b82f6", // Blue 500
    text: "#e2e2e4",
    text_muted: "#94a3b8",
    success: "#10b981",
    danger: "#ef4444",
    border: "#2d2d35",
};

pub struct Colors {
    pub background: &'static str,
    pub surface: &'static str,
    pub surface_light: &'static str,
    pub primary: &'static str,
    pub primary_hover: &'static str,
    pub secondary: &'static str,
    pub text: &'static str,
    pub text_muted: &'static str,
    pub success: &'static str,
    pub danger: &'static str,
    pub border: &'static str,
}

#[derive(Properties, PartialEq)]
pub struct ThemeProps {
    pub children: Children,
}

#[function_component(ThemeProvider)]
pub fn theme_provider(props: &ThemeProps) -> Html {
    html! {
        <>
            <Global css={css!(
                r#"
                * {
                    box-sizing: border-box;
                    -webkit-font-smoothing: antialiased;
                    -moz-osx-font-smoothing: grayscale;
                }
                body {
                    margin: 0;
                    padding: 0;
                    background-color: ${bg};
                    color: ${text};
                    font-family: 'Inter', system-ui, -apple-system, sans-serif;
                }
                h1, h2, h3, h4, h5, h6 {
                    font-family: 'Outfit', sans-serif;
                    margin: 0;
                }
                a {
                    color: inherit;
                    text-decoration: none;
                }
                button {
                    font-family: inherit;
                }
                ::-webkit-scrollbar {
                    width: 8px;
                    height: 8px;
                }
                ::-webkit-scrollbar-track {
                    background: transparent;
                }
                ::-webkit-scrollbar-thumb {
                    background: ${border};
                    border-radius: 4px;
                }
                ::-webkit-scrollbar-thumb:hover {
                    background: ${surface_light};
                }
                "#,
                bg = COLORS.background,
                text = COLORS.text,
                border = COLORS.border,
                surface_light = COLORS.surface_light
            )} />
            { props.children.clone() }
        </>
    }
}
