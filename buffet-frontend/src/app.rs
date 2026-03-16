use yew::prelude::*;
use yew_router::prelude::*;

use crate::pages::backtest_detail::BacktestDetail;
use crate::pages::backtests::Backtests;
use crate::pages::dashboard::Dashboard;
use crate::pages::strategies::Strategies;
use crate::pages::strategy_detail::StrategyDetail;
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
        Route::StrategyDetail { id } => html! { <StrategyDetail {id} /> },
        Route::Strategies => html! { <Strategies /> },
        Route::BacktestDetail { id } => html! { <BacktestDetail {id} /> },
        Route::Backtests => html! { <Backtests /> },
        Route::Settings => html! { <h1>{ "Settings" }</h1> },
        Route::NotFound => html! { <h1>{ "404 Not Found" }</h1> },
    }
}
