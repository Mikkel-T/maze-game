pub mod maze;

use bevy::{
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
};
use maze::Direction;

const HEIGHT: f32 = 600.;
const WIDTH: f32 = 900.;

const PLAYER_SPEED: f32 = 75.;

const TIME_STEP: f32 = 1. / 60.;

const MAZE_BORDER_WIDTH: f32 = 6.;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Maze game".to_string(),
                resizable: false,
                resolution: (WIDTH, HEIGHT).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::WHITE))
        .add_startup_system(setup)
        .add_system(move_player.in_schedule(CoreSchedule::FixedUpdate))
        .insert_resource(FixedTime::new_from_secs(TIME_STEP))
        .add_system(bevy::window::close_on_esc)
        .run();
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Collider;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    let size = 11;

    let m = maze::Maze::new(size);

    let coord_size = (HEIGHT - MAZE_BORDER_WIDTH * (size as f32 + 1.)) / size as f32;

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::RED,
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(0., 0., 0.),
                scale: Vec3::new(coord_size / 2., coord_size / 2., 1.),
                ..default()
            },
            ..default()
        },
        Player,
    ));

    for (i, row) in m.path.iter().enumerate() {
        for (j, cell) in row.iter().enumerate() {
            let x = get_cell_coord(coord_size, j);
            let y = -get_cell_coord(coord_size, i);

            let mut dirs = vec![
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West,
            ];

            dirs.retain(|e| !cell.directions.contains(e));

            for dir in dirs {
                let d = coord_size / 2. + MAZE_BORDER_WIDTH / 2.;
                let mut dx = 0.;
                let mut dy = 0.;
                match dir {
                    Direction::North => dy = d,
                    Direction::South => dy = -d,
                    Direction::East => dx = d,
                    Direction::West => dx = -d,
                };

                let scale = match dir {
                    Direction::North | Direction::South => {
                        Vec3::new(coord_size + 2. * MAZE_BORDER_WIDTH, MAZE_BORDER_WIDTH, 1.)
                    }
                    Direction::East | Direction::West => {
                        Vec3::new(MAZE_BORDER_WIDTH, coord_size + 2. * MAZE_BORDER_WIDTH, 1.)
                    }
                };

                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: Color::BLACK,
                            ..default()
                        },
                        transform: Transform {
                            translation: Vec3::new(x + dx, y + dy, 0.),
                            scale,
                            ..default()
                        },
                        ..default()
                    },
                    Collider,
                ));
            }
        }
    }
}

fn get_cell_coord(coord_size: f32, i: usize) -> f32 {
    MAZE_BORDER_WIDTH + (coord_size + MAZE_BORDER_WIDTH) * i as f32 - (HEIGHT - coord_size) / 2.
}

fn move_player(
    keyboard_input: Res<Input<KeyCode>>,
    mut player_query: Query<&mut Transform, With<Player>>,
    collider_query: Query<&Transform, (With<Collider>, Without<Player>)>,
) {
    let mut player_transform = player_query.single_mut();
    let mut direction = Vec3::new(0., 0., 0.);

    if keyboard_input.pressed(KeyCode::W) || keyboard_input.pressed(KeyCode::Up) {
        direction.y += 1.;
    }

    if keyboard_input.pressed(KeyCode::S) || keyboard_input.pressed(KeyCode::Down) {
        direction.y -= 1.;
    }

    if keyboard_input.pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::Left) {
        direction.x -= 1.;
    }

    if keyboard_input.pressed(KeyCode::D) || keyboard_input.pressed(KeyCode::Right) {
        direction.x += 1.;
    }

    for transform in &collider_query {
        let collision_x = collide(
            transform.translation,
            transform.scale.truncate(),
            player_transform.translation
                + (Vec3::new(direction.x, 0., 0.) * TIME_STEP * PLAYER_SPEED),
            player_transform.scale.truncate(),
        );

        let collision_y = collide(
            transform.translation,
            transform.scale.truncate(),
            player_transform.translation
                + (Vec3::new(0., direction.y, 0.) * TIME_STEP * PLAYER_SPEED),
            player_transform.scale.truncate(),
        );

        if let Some(cx) = collision_x {
            if direction.x < 0. && cx == Collision::Left {
                direction.x = 0.;
                player_transform.translation.x = transform.translation.x
                    + transform.scale.x / 2.
                    + player_transform.scale.x / 2.;
            }

            if direction.x > 0. && cx == Collision::Right {
                direction.x = 0.;
                player_transform.translation.x = transform.translation.x
                    - transform.scale.x / 2.
                    - player_transform.scale.x / 2.;
            }
        }

        if let Some(cy) = collision_y {
            if direction.y < 0. && cy == Collision::Bottom {
                direction.y = 0.;
                player_transform.translation.y = transform.translation.y
                    + transform.scale.y / 2.
                    + player_transform.scale.y / 2.;
            }

            if direction.y > 0. && cy == Collision::Top {
                direction.y = 0.;
                player_transform.translation.y = transform.translation.y
                    - transform.scale.y / 2.
                    - player_transform.scale.y / 2.;
            }
        }
    }

    player_transform.translation += direction * TIME_STEP * PLAYER_SPEED;
}
