pub mod maze;

use bevy::{prelude::*, time::Stopwatch};

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum GameState {
    #[default]
    Menu,
    Game,
    EndGame,
}

#[derive(Resource)]
struct MazeState {
    stopwatch: Stopwatch,
    size: usize,
}

const HEIGHT: f32 = 600.;
const WIDTH: f32 = 1000.;

const TIME_STEP: f32 = 1. / 60.;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Maze game".to_string(),
                        resizable: false,
                        resolution: (WIDTH, HEIGHT).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .insert_resource(ClearColor(Color::WHITE))
        .add_state::<GameState>()
        .insert_resource(MazeState {
            stopwatch: Stopwatch::new(),
            size: 0,
        })
        .add_plugin(menu::MenuPlugin)
        .add_plugin(game::GamePlugin)
        .add_plugin(endscreen::EndScreenPlugin)
        .add_startup_system(setup)
        .insert_resource(FixedTime::new_from_secs(TIME_STEP))
        .add_system(bevy::window::close_on_esc)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

mod menu {
    use super::{despawn_screen, GameState, MazeState};
    use bevy::prelude::*;

    pub struct MenuPlugin;

    impl Plugin for MenuPlugin {
        fn build(&self, app: &mut App) {
            app.add_system(menu_setup.in_schedule(OnEnter(GameState::Menu)))
                .add_system(button_system.in_set(OnUpdate(GameState::Menu)))
                .add_system(despawn_screen::<OnMenuScreen>.in_schedule(OnExit(GameState::Menu)));
        }
    }

    #[derive(Component)]
    struct OnMenuScreen;

    #[derive(Component)]
    struct MazeSize(usize);

    const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
    const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
    const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);
    const TEXT_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);

    #[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
    enum Difficulty {
        Easy,
        Medium,
        Hard,
    }

    fn button_system(
        mut interaction_query: Query<
            (&Interaction, &mut BackgroundColor, &MazeSize),
            (Changed<Interaction>, With<Button>),
        >,
        mut maze_state: ResMut<MazeState>,
        mut game_state: ResMut<NextState<GameState>>,
    ) {
        for (interaction, mut color, size) in &mut interaction_query {
            *color = match *interaction {
                Interaction::Clicked => PRESSED_BUTTON.into(),
                Interaction::Hovered => HOVERED_BUTTON.into(),
                Interaction::None => NORMAL_BUTTON.into(),
            };

            if *interaction == Interaction::Clicked {
                maze_state.size = size.0;
                maze_state.stopwatch.reset();
                game_state.set(GameState::Game);
            }
        }
    }

    fn menu_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
        commands
            .spawn((
                NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    background_color: Color::CRIMSON.into(),
                    ..default()
                },
                OnMenuScreen,
            ))
            .with_children(|parent| {
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        ..default()
                    })
                    .with_children(|parent| {
                        parent.spawn(TextBundle::from_section(
                            "Maze game",
                            TextStyle {
                                font: asset_server.load("fonts/PixeloidSansBold.ttf"),
                                font_size: 80.,
                                color: TEXT_COLOR,
                            },
                        ));

                        parent.spawn(TextBundle::from_section(
                            "Choose difficulty",
                            TextStyle {
                                font: asset_server.load("fonts/PixeloidSansBold.ttf"),
                                font_size: 40.,
                                color: TEXT_COLOR,
                            },
                        ));

                        for (difficulty, size) in [
                            (Difficulty::Easy, 11),
                            (Difficulty::Medium, 21),
                            (Difficulty::Hard, 31),
                        ] {
                            parent
                                .spawn(ButtonBundle {
                                    style: Style {
                                        size: Size::new(Val::Px(200.), Val::Px(65.)),
                                        margin: UiRect::all(Val::Px(20.0)),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        flex_direction: FlexDirection::Column,
                                        ..default()
                                    },
                                    background_color: BackgroundColor(Color::rgb(0.15, 0.15, 0.15)),
                                    ..default()
                                })
                                .insert(MazeSize(size))
                                .with_children(|parent| {
                                    parent.spawn(TextBundle::from_section(
                                        format!("{difficulty:?}"),
                                        TextStyle {
                                            font: asset_server.load("fonts/PixeloidSansBold.ttf"),
                                            font_size: 30.,
                                            color: TEXT_COLOR,
                                        },
                                    ));
                                    parent.spawn(TextBundle::from_section(
                                        format!("{size} x {size}"),
                                        TextStyle {
                                            font: asset_server.load("fonts/PixeloidSansBold.ttf"),
                                            font_size: 20.,
                                            color: TEXT_COLOR,
                                        },
                                    ));
                                });
                        }
                    });
            });
    }
}

mod game {
    use super::{despawn_screen, GameState, MazeState, HEIGHT, TIME_STEP, WIDTH};
    use crate::maze::{Direction, Maze};
    use bevy::{
        prelude::*,
        sprite::{
            collide_aabb::{collide, Collision},
            Anchor,
        },
    };

    pub struct GamePlugin;

    impl Plugin for GamePlugin {
        fn build(&self, app: &mut App) {
            app.add_system(menu_setup.in_schedule(OnEnter(GameState::Game)))
                .add_systems(
                    (
                        move_player,
                        coin_check.after(move_player),
                        time_check.after(move_player),
                    )
                        .distributive_run_if(in_state(GameState::Game))
                        .in_schedule(CoreSchedule::FixedUpdate),
                )
                .add_system(despawn_screen::<OnGameScreen>.in_schedule(OnExit(GameState::Game)));
        }
    }

    #[derive(Component)]
    struct OnGameScreen;

    const PLAYER_SPEED: f32 = 6.;

    const MAZE_BORDER_WIDTH: f32 = 3.;

    const WALL_COLOR: Color = Color::BLACK;

    #[derive(Component)]
    struct Player;

    #[derive(Component)]
    struct Collider;

    #[derive(Component)]
    struct Start;

    #[derive(Component)]
    struct End;

    #[derive(Component)]
    struct Coin;

    #[derive(Component)]
    struct EndGate;

    #[derive(Component)]
    struct TimerBoard;

    fn menu_setup(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        maze_state: Res<MazeState>,
    ) {
        let size = maze_state.size;
        let coins = (size + 9) / 4;

        let m = Maze::new(size, coins);

        let coord_size = (HEIGHT - MAZE_BORDER_WIDTH * (size as f32 + 1.)) / size as f32;

        // Spawn background
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::CRIMSON,
                    ..default()
                },
                transform: Transform {
                    scale: Vec3::new(WIDTH, HEIGHT, -1.),
                    ..default()
                },
                ..default()
            },
            OnGameScreen,
        ));

        // Spawn player
        commands.spawn((
            SpriteBundle {
                texture: asset_server.load("images/player.png"),
                sprite: Sprite {
                    custom_size: Some(Vec2::ONE),
                    ..default()
                },
                transform: Transform {
                    translation: Vec3::new(-300. - coord_size, 0., 1.),
                    scale: Vec3::new(coord_size / 2., coord_size / 2., 1.),
                    ..default()
                },
                ..default()
            },
            Player,
            OnGameScreen,
        ));

        // Spawn walls and coins for the maze
        for (i, row) in m.path.iter().enumerate() {
            for (j, cell) in row.iter().enumerate() {
                let x = get_cell_coord(coord_size, j);
                let y = -get_cell_coord(coord_size, i);

                if cell.coin {
                    commands.spawn((
                        SpriteBundle {
                            texture: asset_server.load("images/coin.png"),
                            sprite: Sprite {
                                custom_size: Some(Vec2::ONE),
                                ..default()
                            },
                            transform: Transform {
                                translation: Vec3::new(x, y, 0.),
                                scale: Vec3::new(coord_size * 0.75, coord_size * 0.75, 1.),
                                ..default()
                            },
                            ..default()
                        },
                        Coin,
                        OnGameScreen,
                    ));
                }

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
                                color: WALL_COLOR,
                                ..default()
                            },
                            transform: Transform {
                                translation: Vec3::new(x + dx, y + dy, 1.),
                                scale,
                                ..default()
                            },
                            ..default()
                        },
                        Collider,
                        OnGameScreen,
                    ));
                }
            }
        }

        // Spawn start
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::LIME_GREEN,
                    ..default()
                },
                transform: Transform {
                    translation: Vec3::new(-300. - coord_size, 0., 0.),
                    scale: Vec3::new(coord_size * 2., coord_size * 2., 0.),
                    ..default()
                },
                ..default()
            },
            Start,
            OnGameScreen,
        ));

        // Spawn end
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::TOMATO,
                    ..default()
                },
                transform: Transform {
                    translation: Vec3::new(300. + coord_size, 0., 0.),
                    scale: Vec3::new(coord_size * 2., coord_size * 2., 0.),
                    ..default()
                },
                ..default()
            },
            End,
            OnGameScreen,
        ));

        // Spawn end gate
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: WALL_COLOR,
                    ..default()
                },
                transform: Transform {
                    translation: Vec3::new(300. - MAZE_BORDER_WIDTH / 2., 0., 1.),
                    scale: Vec3::new(MAZE_BORDER_WIDTH, coord_size, 0.),
                    ..default()
                },
                ..default()
            },
            Collider,
            EndGate,
            OnGameScreen,
        ));

        // Spawn timer
        commands.spawn((
            Text2dBundle {
                text: Text::from_section(
                    "0.00",
                    TextStyle {
                        font: asset_server.load("fonts/PixeloidSansBold.ttf"),
                        font_size: 40.,
                        color: WALL_COLOR,
                    },
                ),
                transform: Transform::from_translation(Vec3::new(WIDTH / 2., HEIGHT / 2., 1.)),
                text_anchor: Anchor::TopRight,
                ..default()
            },
            TimerBoard,
            OnGameScreen,
        ));

        // Spawn walls surrounding the start and end
        for i in [-1., 1.] {
            // Spawn left and right walls
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: WALL_COLOR,
                        ..default()
                    },
                    transform: Transform {
                        translation: Vec3::new(i * (300. + coord_size * 2.), 0., 1.),
                        scale: Vec3::new(
                            MAZE_BORDER_WIDTH,
                            coord_size * 2. + MAZE_BORDER_WIDTH,
                            0.,
                        ),
                        ..default()
                    },
                    ..default()
                },
                Collider,
                OnGameScreen,
            ));

            // Spawn upper and lower walls
            for j in [-1., 1.] {
                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: WALL_COLOR,
                            ..default()
                        },
                        transform: Transform {
                            translation: Vec3::new(i * (300. + coord_size), j * coord_size, 1.),
                            scale: Vec3::new(
                                coord_size * 2. + MAZE_BORDER_WIDTH,
                                MAZE_BORDER_WIDTH,
                                0.,
                            ),
                            ..default()
                        },
                        ..default()
                    },
                    Collider,
                    OnGameScreen,
                ));
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
        let player_scale = player_transform.scale;
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
                player_scale.truncate(),
            );

            let collision_y = collide(
                transform.translation,
                transform.scale.truncate(),
                player_transform.translation
                    + (Vec3::new(0., direction.y, 0.) * TIME_STEP * PLAYER_SPEED),
                player_scale.truncate(),
            );

            if let Some(cx) = collision_x {
                if direction.x < 0. && cx == Collision::Left {
                    direction.x = 0.;
                    player_transform.translation.x =
                        transform.translation.x + transform.scale.x / 2. + player_scale.x / 2.;
                }

                if direction.x > 0. && cx == Collision::Right {
                    direction.x = 0.;
                    player_transform.translation.x =
                        transform.translation.x - transform.scale.x / 2. - player_scale.x / 2.;
                }
            }

            if let Some(cy) = collision_y {
                if direction.y < 0. && cy == Collision::Bottom {
                    direction.y = 0.;
                    player_transform.translation.y =
                        transform.translation.y + transform.scale.y / 2. + player_scale.y / 2.;
                }

                if direction.y > 0. && cy == Collision::Top {
                    direction.y = 0.;
                    player_transform.translation.y =
                        transform.translation.y - transform.scale.y / 2. - player_scale.y / 2.;
                }
            }
        }

        player_transform.translation += direction * TIME_STEP * PLAYER_SPEED * player_scale;
    }

    fn coin_check(
        mut commands: Commands,
        player_query: Query<&Transform, With<Player>>,
        coin_query: Query<(Entity, &Transform), With<Coin>>,
        end_gate_query: Query<Entity, With<EndGate>>,
    ) {
        let player_transform = player_query.single();

        for (entity, transform) in coin_query.iter() {
            if collide(
                transform.translation,
                transform.scale.truncate(),
                player_transform.translation,
                player_transform.scale.truncate(),
            )
            .is_some()
            {
                commands.entity(entity).despawn();
                if coin_query.iter().count() == 1 {
                    commands.entity(end_gate_query.single()).despawn();
                }
            }
        }
    }

    fn time_check(
        mut scoreboard_query: Query<&mut Text, With<TimerBoard>>,
        time: Res<Time>,
        start_query: Query<&Transform, With<Start>>,
        end_query: Query<&Transform, With<End>>,
        player_query: Query<&Transform, With<Player>>,
        mut maze_state: ResMut<MazeState>,
        mut game_state: ResMut<NextState<GameState>>,
    ) {
        let mut text = scoreboard_query.single_mut();

        if !(maze_state.stopwatch.paused()
            || collide(
                player_query.single().translation,
                player_query.single().scale.truncate(),
                start_query.single().translation,
                start_query.single().scale.truncate(),
            )
            .is_some()
                && maze_state.stopwatch.elapsed_secs() == 0.)
        {
            maze_state.stopwatch.tick(time.delta());
        }

        if collide(
            player_query.single().translation,
            player_query.single().scale.truncate(),
            end_query.single().translation,
            end_query.single().scale.truncate(),
        )
        .is_some()
        {
            maze_state.stopwatch.pause();
            game_state.set(GameState::EndGame);
        }

        text.sections[0].value = format!("{:.3}", maze_state.stopwatch.elapsed_secs());
    }
}

mod endscreen {
    use super::{despawn_screen, GameState, MazeState};
    use bevy::prelude::*;

    pub struct EndScreenPlugin;

    impl Plugin for EndScreenPlugin {
        fn build(&self, app: &mut App) {
            app.add_system(endscreen_setup.in_schedule(OnEnter(GameState::EndGame)))
                .add_system(button_system.in_set(OnUpdate(GameState::EndGame)))
                .add_system(despawn_screen::<OnEndScreen>.in_schedule(OnExit(GameState::EndGame)));
        }
    }

    #[derive(Component)]
    struct OnEndScreen;

    const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
    const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
    const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);
    const TEXT_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);

    fn button_system(
        mut interaction_query: Query<
            (&Interaction, &mut BackgroundColor),
            (Changed<Interaction>, With<Button>),
        >,
        mut game_state: ResMut<NextState<GameState>>,
    ) {
        for (interaction, mut color) in &mut interaction_query {
            *color = match *interaction {
                Interaction::Clicked => PRESSED_BUTTON.into(),
                Interaction::Hovered => HOVERED_BUTTON.into(),
                Interaction::None => NORMAL_BUTTON.into(),
            };

            if *interaction == Interaction::Clicked {
                game_state.set(GameState::Menu);
            }
        }
    }

    fn endscreen_setup(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        maze_state: Res<MazeState>,
    ) {
        commands
            .spawn((
                NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    background_color: Color::CRIMSON.into(),
                    ..default()
                },
                OnEndScreen,
            ))
            .with_children(|parent| {
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        ..default()
                    })
                    .with_children(|parent| {
                        parent.spawn(TextBundle::from_section(
                            "Congratulations!",
                            TextStyle {
                                font: asset_server.load("fonts/PixeloidSansBold.ttf"),
                                font_size: 60.,
                                color: TEXT_COLOR,
                            },
                        ));
                        parent.spawn(TextBundle::from_section(
                            format!(
                                "You just completed a {} x {} maze in",
                                maze_state.size, maze_state.size
                            ),
                            TextStyle {
                                font: asset_server.load("fonts/PixeloidSansBold.ttf"),
                                font_size: 30.,
                                color: TEXT_COLOR,
                            },
                        ));

                        parent.spawn(TextBundle::from_section(
                            format!("{:.3} seconds!", maze_state.stopwatch.elapsed_secs()),
                            TextStyle {
                                font: asset_server.load("fonts/PixeloidSansBold.ttf"),
                                font_size: 40.,
                                color: TEXT_COLOR,
                            },
                        ));

                        parent
                            .spawn(ButtonBundle {
                                style: Style {
                                    size: Size::new(Val::Px(300.), Val::Px(65.)),
                                    margin: UiRect::all(Val::Px(20.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    flex_direction: FlexDirection::Column,
                                    ..default()
                                },
                                background_color: BackgroundColor(Color::rgb(0.15, 0.15, 0.15)),
                                ..default()
                            })
                            .with_children(|parent| {
                                parent.spawn(TextBundle::from_section(
                                    "Back to start",
                                    TextStyle {
                                        font: asset_server.load("fonts/PixeloidSansBold.ttf"),
                                        font_size: 30.,
                                        color: TEXT_COLOR,
                                    },
                                ));
                            });
                    });
            });
    }
}

fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}
