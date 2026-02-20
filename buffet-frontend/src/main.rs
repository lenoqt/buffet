mod app;
mod components;
mod pages;
mod routes;
mod services;
mod theme;

fn main() {
    yew::Renderer::<app::App>::new().render();
}
