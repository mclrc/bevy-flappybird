use std::{f32::consts::PI, time::Duration};

use bevy::{
    prelude::*,
    sprite::{collide_aabb::collide, Anchor},
    time::common_conditions::on_fixed_timer,
};
use rand::Rng;

const SCALE: Vec3 = Vec3::new(3., 3., 3.);
const SPEED: f32 = 4.5;
const PIPE_INTERVAL: u64 = 1;
const PIPE_GAP: f32 = 150.;
const PIPE_HEIGHT: f32 = 160. * 3.;
const FLAP_SPEED: f32 = 4.5;
const WINDOW_WIDTH: f32 = 400.;
const WINDOW_HEIGHT: f32 = 700.;
const MIN_PIPE_OFFSET: f32 = 100.;
const PIPE_WIDTH: f32 = 26. * 3.;
const FLOOR_SEGMENT_WIDTH: f32 = 168. * 3.;
const FLOOR_HEIGHT: f32 = 50.;
const BACKGROUND_SEGMENT_WIDTH: f32 = 144. * 3.;

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum GameState {
    #[default]
    Menu,
    InGame,
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Pipe;

#[derive(Component)]
struct Floor;

#[derive(Component)]
struct Velocity {
    x: f32,
    y: f32,
}

#[derive(Component)]
struct Mass;

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

#[derive(Component)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component)]
struct InfiniteScrolling {
    segment_width: f32,
    speed: f32,
}

fn spawn_player(
    mut commands: Commands,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    asset_server: Res<AssetServer>,
) {
    let initial_position = Transform::from_xyz(-150., 0., 0.).with_scale(SCALE.clone());

    let texture_handle = asset_server.load("bird.png");
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(20., 20.), 4, 1, None, None);

    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    commands.spawn((
        Player,
        Velocity { x: 0., y: 0. },
        Mass,
        SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            sprite: TextureAtlasSprite::new(0),
            transform: initial_position,
            ..Default::default()
        },
        AnimationIndices { first: 0, last: 3 },
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
    ));
}

fn flap_system(keyboard_input: Res<Input<KeyCode>>, mut query: Query<&mut Velocity, With<Player>>) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        let mut player_vel = query.single_mut();
        player_vel.y = FLAP_SPEED;
    }
}

fn animate_sprite_system(
    time: Res<Time>,
    mut query: Query<(
        &mut TextureAtlasSprite,
        &mut AnimationTimer,
        &AnimationIndices,
    )>,
) {
    for (mut sprite, mut timer, indices) in query.iter_mut() {
        timer.tick(time.delta());
        if timer.finished() {
            sprite.index = if sprite.index == indices.last {
                indices.first
            } else {
                sprite.index + 1
            }
        }
    }
}

fn tilt_with_vel_system(mut query: Query<(&mut Transform, &Velocity), With<Player>>) {
    for (mut transform, velocity) in query.iter_mut() {
        let angle = velocity.y / 5. * PI / 4.;
        transform.rotation = Quat::from_rotation_z(angle);
    }
}
fn movement_system(mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation.x += velocity.x;
        transform.translation.y += velocity.y;
    }
}

fn spawn_pipes_system(mut commands: Commands, asset: Res<AssetServer>) {
    let texture: Handle<Image> = asset.load("pipe.png");

    let gap_bottom = rand::thread_rng().gen_range(
        (-WINDOW_HEIGHT / 2. + MIN_PIPE_OFFSET)..(WINDOW_HEIGHT / 2. - MIN_PIPE_OFFSET - PIPE_GAP),
    );
    let gap_top = gap_bottom + PIPE_GAP;

    commands.spawn((
        Pipe,
        Velocity { x: -SPEED, y: 0. },
        SpriteBundle {
            texture: texture.clone(),
            transform: Transform::from_xyz(400., gap_top + PIPE_HEIGHT, 0.)
                .with_scale(SCALE.clone()),
            sprite: Sprite {
                flip_y: true,
                anchor: Anchor::TopLeft,
                ..Default::default()
            },
            ..Default::default()
        },
    ));
    commands.spawn((
        Pipe,
        Velocity { x: -SPEED, y: 0. },
        SpriteBundle {
            transform: Transform::from_xyz(400., gap_bottom, 0.).with_scale(SCALE.clone()),
            texture,
            sprite: Sprite {
                anchor: Anchor::TopLeft,
                ..Default::default()
            },
            ..Default::default()
        },
    ));
}

fn remove_pipes_system(mut commands: Commands, mut query: Query<(Entity, &Transform), With<Pipe>>) {
    for (entity, transform) in query.iter_mut() {
        if transform.translation.x < -WINDOW_WIDTH / 2. - PIPE_WIDTH {
            commands.entity(entity).despawn();
        }
    }
}

fn spawn_floor_system(mut commands: Commands, asset: Res<AssetServer>) {
    commands.spawn((
        Floor,
        InfiniteScrolling {
            segment_width: FLOOR_SEGMENT_WIDTH,
            speed: -SPEED,
        },
        SpriteBundle {
            texture: asset.load("floor.png"),
            transform: Transform::from_xyz(
                -WINDOW_WIDTH / 2. + FLOOR_SEGMENT_WIDTH,
                -WINDOW_HEIGHT / 2. + FLOOR_HEIGHT,
                10.,
            )
            .with_scale(SCALE.clone()),
            sprite: Sprite {
                anchor: Anchor::TopLeft,
                ..Default::default()
            },
            ..Default::default()
        },
    ));
    commands.spawn((
        Floor,
        InfiniteScrolling {
            segment_width: FLOOR_SEGMENT_WIDTH,
            speed: -SPEED,
        },
        SpriteBundle {
            texture: asset.load("floor.png"),
            transform: Transform::from_xyz(
                -WINDOW_WIDTH / 2.,
                -WINDOW_HEIGHT / 2. + FLOOR_HEIGHT,
                10.,
            )
            .with_scale(SCALE.clone()),
            sprite: Sprite {
                anchor: Anchor::TopLeft,
                ..Default::default()
            },
            ..Default::default()
        },
    ));
}

fn spawn_background_system(mut commands: Commands, asset: Res<AssetServer>) {
    commands.spawn((
        Floor,
        InfiniteScrolling {
            segment_width: BACKGROUND_SEGMENT_WIDTH,
            speed: -SPEED * 0.2,
        },
        SpriteBundle {
            texture: asset.load("bg.png"),
            transform: Transform::from_xyz(
                -WINDOW_WIDTH / 2. + BACKGROUND_SEGMENT_WIDTH,
                WINDOW_HEIGHT / 2.,
                0.,
            )
            .with_scale(SCALE.clone()),
            sprite: Sprite {
                anchor: Anchor::TopLeft,
                ..Default::default()
            },
            ..Default::default()
        },
    ));
    commands.spawn((
        Floor,
        InfiniteScrolling {
            segment_width: BACKGROUND_SEGMENT_WIDTH,
            speed: -SPEED * 0.2,
        },
        SpriteBundle {
            texture: asset.load("bg.png"),
            transform: Transform::from_xyz(-WINDOW_WIDTH / 2., WINDOW_HEIGHT / 2., 0.)
                .with_scale(SCALE.clone()),
            sprite: Sprite {
                anchor: Anchor::TopLeft,
                ..Default::default()
            },
            ..Default::default()
        },
    ));
}

fn infinite_scrolling_system(mut query: Query<(&mut Transform, &InfiniteScrolling)>) {
    for (
        mut transform,
        InfiniteScrolling {
            segment_width,
            speed,
        },
    ) in query.iter_mut()
    {
        transform.translation.x += speed;
        if transform.translation.x < -WINDOW_WIDTH / 2. - segment_width {
            transform.translation.x += segment_width * 2.;
        }
    }
}

fn gravity_system(time: Res<Time>, mut query: Query<(&mut Velocity, &Mass)>) {
    let acceleration = 9.8 * time.delta_seconds() as f32;

    for (mut velocity, ..) in query.iter_mut() {
        velocity.y -= acceleration;
    }
}

fn game_over_system(
    mut next_state: ResMut<NextState<GameState>>,
    mut player_query: Query<(&mut Transform, &mut Velocity), With<Player>>,
    pipes_query: Query<&Transform, (Without<Player>, With<Pipe>)>,
) {
    let mut game_over = false;

    let (mut transform, mut velocity) = player_query.single_mut();

    if transform.translation.y < -WINDOW_HEIGHT / 2. + FLOOR_HEIGHT {
        game_over = true;
    }

    for pipe in pipes_query.iter() {
        if collide(
            transform.translation,
            Vec2::new(45., 45.),
            pipe.translation + Vec3::new(PIPE_WIDTH / 2., -PIPE_HEIGHT / 2., 0.),
            Vec2::new(PIPE_WIDTH, PIPE_HEIGHT),
        )
        .is_some()
        {
            game_over = true;
            break;
        }
    }

    if game_over {
        next_state.set(GameState::Menu);
    };
}

fn setup_menu_system(
    mut pipes: Query<Entity, (With<Pipe>, Without<Player>)>,
    mut player: Query<(&mut Transform, &mut Velocity), With<Player>>,
    mut commands: Commands,
) {
    let (mut transform, mut velocity) = player.single_mut();
    velocity.y = 0.;
    transform.translation.y = 0.;

    pipes.iter_mut().for_each(|entity| {
        commands.entity(entity).despawn();
    });
}

fn start_game_system(
    mut next_state: ResMut<NextState<GameState>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        next_state.set(GameState::InGame);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Flappy Bird".into(),
                        resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                        resizable: false,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_state::<GameState>()
        .add_startup_system(setup)
        .add_startup_system(spawn_floor_system)
        .add_startup_system(spawn_background_system)
        .add_startup_system(spawn_player)
        .add_system(infinite_scrolling_system)
        .add_system(start_game_system.run_if(in_state(GameState::Menu)))
        .add_system(setup_menu_system.in_schedule(OnEnter(GameState::Menu)))
        .add_systems((
            spawn_pipes_system
                .in_schedule(CoreSchedule::FixedUpdate)
                .run_if(on_fixed_timer(Duration::from_secs(PIPE_INTERVAL)))
                .run_if(in_state(GameState::InGame)),
            remove_pipes_system
                .in_schedule(CoreSchedule::FixedUpdate)
                .run_if(on_fixed_timer(Duration::from_secs(PIPE_INTERVAL)))
                .run_if(in_state(GameState::InGame)),
            flap_system.run_if(in_state(GameState::InGame)),
            gravity_system.run_if(in_state(GameState::InGame)),
            game_over_system
                .in_schedule(CoreSchedule::FixedUpdate)
                .run_if(on_fixed_timer(Duration::from_millis(1000 / 30)))
                .run_if(in_state(GameState::InGame)),
        ))
        .add_system(tilt_with_vel_system)
        .add_system(movement_system)
        .add_system(animate_sprite_system)
        .run();
}
