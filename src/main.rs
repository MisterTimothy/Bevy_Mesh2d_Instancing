mod vertex_buffer;

use bevy::prelude::*;

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(vertex_buffer::InstancingPlugin)
        .run()
}
