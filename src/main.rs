use amethyst::{
    assets::{AssetStorage, Loader, Handle},
    core::{
        math as na,
        transform::{TransformBundle, Transform},
        Named
    },
    ecs::{Entity, Component, System, Join, VecStorage, NullStorage},
    ecs::prelude::{
        Read,
        ReadStorage,
        WriteStorage,
        Resources,
        SystemData
    },
    input::{InputBundle, InputHandler, StringBindings},
    prelude::*,
    renderer::{
        plugins::{RenderFlat2D, RenderToWindow},
        types::DefaultBackend,
        camera::Projection,
        Camera,
        ImageFormat,
        SpriteSheetFormat,
        RenderingBundle,
        SpriteRender, SpriteSheet,
        Texture, Transparent
    },
    utils::application_root_dir,
    window::ScreenDimensions
};

const SCALE_FACTOR: f32 = 3.;

fn load_sprite_sheet(world: &World) -> Handle<SpriteSheet> {
    let texture_handle = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        loader.load("sprite_sheet.png", ImageFormat::default(), (), &texture_storage)
    };
    let loader = world.read_resource::<Loader>();
    let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
    loader.load(
        "sprite_sheet.ron",
        SpriteSheetFormat(texture_handle),
        (),
        &sprite_sheet_store
    )
}

fn init_player_sprite(world: &mut World, sprite_sheet_handle: &Handle<SpriteSheet>) {
    let mut sprite_transform = Transform::default();
    sprite_transform.set_translation_xyz(30., 30., 0.);
    let sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle.clone(),
        sprite_number: 0
    };
    world.create_entity()
        .with(sprite_render)
        .with(sprite_transform)
        .with(Transparent)
        .with(ActorType::Player)
        .build();
}

fn init_enemy_sprite(world: &mut World, sprite_sheet_handle: &Handle<SpriteSheet>) {
    let mut sprite_transform = Transform::default();
    sprite_transform.set_translation_xyz(100., 30., -1.);
    sprite_transform.set_rotation_y_axis(std::f32::consts::PI);
    let sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle.clone(),
        sprite_number: 1
    };
    world.create_entity()
        .with(sprite_render)
        .with(sprite_transform)
        .with(Transparent)
        .build();
}

fn init_camera(world: &mut World) {
    let (width, height) = {
        let dim = world.read_resource::<ScreenDimensions>();
        (dim.width()/SCALE_FACTOR, dim.height()/SCALE_FACTOR)
    };

    let mut transform = Transform::default();
    transform.set_translation_xyz(width/2., height/2., 1.);
    let camera = Camera::standard_2d(width, height);
    world.create_entity()
        .with(camera)
        .with(transform)
        .with(ActorType::Camera)
        .build();
}

#[derive(Default, Debug)]
struct MyState;

impl SimpleState for MyState {
    fn on_start(&mut self, mut data: StateData<'_, GameData<'_, '_>>) {
        let sprite_sheet_handle = load_sprite_sheet(&data.world);
        init_player_sprite(&mut data.world, &sprite_sheet_handle);
        init_enemy_sprite(&mut data.world, &sprite_sheet_handle);
        init_camera(&mut data.world);
    }
}

#[derive(Debug)]
enum ActorType {
    Player,
    Camera
}
impl Component for ActorType {
    type Storage = VecStorage<Self>;
}

struct MovementSystem;
impl <'a> System<'a> for MovementSystem {
    type SystemData = (
        ReadStorage<'a, ActorType>,
        WriteStorage<'a, Transform>,
        Read<'a, InputHandler<StringBindings>>
    );
    fn run(&mut self, (actor_types, mut transforms, input): Self::SystemData) {
        let (cam_move_x, cam_move_y, player_move_x, player_move_y) = (
            input.axis_value("l_x").unwrap(),
            input.axis_value("l_y").unwrap(),
            input.axis_value("r_x").unwrap(),
            input.axis_value("r_y").unwrap()
        );
        for (actor_type, transform) in (&actor_types, &mut transforms).join() {
            match actor_type {
                ActorType::Player => {
                    transform.prepend_translation_x(player_move_x as f32 * 5.0);
                    transform.prepend_translation_y(player_move_y as f32 * 5.0);
                }
                ActorType::Camera => {
                    transform.prepend_translation_x(cam_move_x as f32 * 5.0);
                    transform.prepend_translation_y(cam_move_y as f32 * 5.0);
                }
            };
        }
    }
}

struct DebugSystem;
impl <'a> System<'a> for DebugSystem {
    type SystemData = (
        ReadStorage<'a, Named>,
        ReadStorage<'a, Transform>
    );
    fn run(&mut self, (names, transforms): Self::SystemData) {
        for (name, transform) in (&names, &transforms).join() {
            let translation = transform.translation();
            println!("entity {} transform ({}, {}, {})", name.name, translation.x, translation.y, translation.z);
        }
    }
}

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
        .with(DebugSystem, "debug_system", &["movement_system"])
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)
                        .with_clear([0.34, 0.36, 0.52, 1.0]),
                )
                .with_plugin(RenderFlat2D::default()),
        )?;

    let mut game = Application::new(assets_dir, MyState, game_data)?;
    game.run();

    Ok(())
}
