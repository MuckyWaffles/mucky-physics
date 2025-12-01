use raylib::prelude::*;

let screenWidth = 1000;
let screenHeight = 600;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(screenWidth, screenHeight)
        .title("Mucky Physics")
        .build();

    while (!rl.window_should_close()) {
        let mut d = rl.begin_drawing(&thread);

        d.clearBackground(.black);

        d.end_drawing(&thread);
    }
}
