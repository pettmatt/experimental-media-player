use std::env;
fn main() {
	env::set_var("RUST_BACKTRACE", "full");
    slint_build::compile("ui/app-window.slint").expect("Slint build failed");
}
