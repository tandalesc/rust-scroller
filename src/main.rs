use amethyst::{
    core::{
        transform::{TransformBundle}
    },
    input::{InputBundle, StringBindings},
    prelude::*,
    renderer::{
        plugins::{RenderFlat2D, RenderToWindow},
        types::DefaultBackend,
        RenderingBundle,
    },
    utils::application_root_dir
};

mod animation;
mod character;
mod tilemap;
mod state;
mod system;
mod hitbox;

use crate::system::{
    MovementSystem,
    PhysicsSystem,
    UpdateCameraSystem
};
use crate::character::{
    PlayerSystem
};
use crate::animation::{
    AnimationSystem
};
use crate::state::{
    LoadMapState
};

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;

    let assets_dir = app_root.join("assets");
    let config_dir = app_root.join("config");
    let display_config_path = config_dir.join("display.ron");

    let game_data = GameDataBuilder::default()
        .with_bundle(TransformBundle::new())?
        .with_bundle(
            InputBundle::<StringBindings>::new()
                .with_bindings_from_file(config_dir.join("input.ron"))?
        )?
        .with(MovementSystem, "movement_system", &[])
        .with(PhysicsSystem, "physics_system", &["movement_system"])
        .with(UpdateCameraSystem, "update_camera_system", &["movement_system"])
        .with(PlayerSystem, "player_system", &["physics_system"])
        .with(AnimationSystem, "animation_system", &["player_system"])
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)
                        .with_clear([0.34, 0.36, 0.52, 1.0]),
                )
                .with_plugin(RenderFlat2D::default()),
        )?;

    let mut game = Application::new(assets_dir, LoadMapState::new(), game_data)?;
    game.run();

    Ok(())
}
