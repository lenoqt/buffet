use yew_router::prelude::*;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Dashboard,
    #[at("/strategies/:id")]
    StrategyDetail { id: String },
    #[at("/strategies")]
    Strategies,
    #[at("/backtests/:id")]
    BacktestDetail { id: String },
    #[at("/backtests")]
    Backtests,
    #[at("/settings")]
    Settings,
    #[not_found]
    #[at("/404")]
    NotFound,
}
