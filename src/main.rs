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

mod gamestate;
mod animation;
mod character;

use crate::gamestate::{
    GameState,
    MovementSystem,
    PhysicsSystem
};
use crate::character::{
    PlayerSystem
};
use crate::animation::{
    AnimationSystem
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
        .with(PhysicsSystem, "physics_system", &[])
        .with(PlayerSystem, "player_system", &[])
        .with(AnimationSystem, "animation_system", &[])
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)
                        .with_clear([0.34, 0.36, 0.52, 1.0]),
                )
                .with_plugin(RenderFlat2D::default()),
        )?;

    let mut game = Application::new(assets_dir, GameState, game_data)?;
    game.run();

    Ok(())
}
