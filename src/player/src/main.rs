#[path = "./player.rs"]
mod player;

fn main() {
	player::main_loop("../media/test.mp4");
}
