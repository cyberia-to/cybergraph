//! studio — the cyberlink composer. One screen: link two particles, move a
//! box, set a valence, add to the draft signal, sign it with the neuron.

mod app;
mod signal;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(app::App);
}
