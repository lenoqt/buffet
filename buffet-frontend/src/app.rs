use yew::prelude::*;
use yew_router::prelude::*;

use crate::pages::backtests::Backtests;
use crate::pages::dashboard::Dashboard;
use crate::pages::strategies::Strategies;
use crate::routes::Route;
use crate::theme::ThemeProvider;

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <ThemeProvider>
            <BrowserRouter>
                <Switch<Route> render={switch} />
            </BrowserRouter>
        </ThemeProvider>
    }
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Dashboard => html! { <Dashboard /> },
        Route::Strategies => html! { <Strategies /> },
        Route::Backtests => html! { <Backtests /> },
        Route::Settings => html! { <h1>{ "Settings" }</h1> },
        Route::NotFound => html! { <h1>{ "404 Not Found" }</h1> },
    }
}
