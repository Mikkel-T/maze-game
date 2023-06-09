use crate::maze::{Direction, Maze};
use crate::utils::vars::{BUTTON_MARGIN, BUTTON_SIZE};
use crate::utils::{
    colors::{TEXT_COLOR, WALL_COLOR},
    despawn_screen,
    vars::{GameState, MazeState, HEIGHT, MAZE_BORDER_WIDTH, PLAYER_SPEED, WIDTH},
};
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
                    coin_count.after(move_player),
                )
                    .distributive_run_if(in_state(GameState::Game)),
            )
            .add_system(button_system.run_if(in_state(GameState::Game)))
            .add_system(despawn_screen::<OnGameScreen>.in_schedule(OnExit(GameState::Game)));
    }
}

#[derive(Component)]
enum ButtonAction {
    Pause,
    Resume,
    Quit,
}

#[derive(Component)]
enum ShowDuring {
    Paused,
    Running,
}

#[derive(Component)]
struct OnGameScreen;

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

#[derive(Component)]
struct CoinCounter;

fn menu_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut maze_state: ResMut<MazeState>,
) {
    let size = maze_state.size;
    let coins = (size + 9) / 4;

    maze_state.coins = coins;

    let path = &maze_state.path;

    let m = match path.clone() {
        Some(path) => path,
        None => Maze::new(size, coins),
    };

    maze_state.path = Some(m.clone());

    let coord_size = (HEIGHT - MAZE_BORDER_WIDTH * (size as f32 + 1.)) / size as f32;

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
        ShowDuring::Running,
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
                    ShowDuring::Running,
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
                    ShowDuring::Running,
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
        ShowDuring::Running,
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
        ShowDuring::Running,
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
        ShowDuring::Running,
    ));

    // Spawn timer
    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                "0.00",
                TextStyle {
                    font: asset_server.load("fonts/PixeloidSansBold.ttf"),
                    font_size: 40.,
                    color: TEXT_COLOR,
                },
            ),
            transform: Transform::from_translation(Vec3::new(WIDTH / 2., HEIGHT / 2., 1.)),
            text_anchor: Anchor::TopRight,
            ..default()
        },
        TimerBoard,
        OnGameScreen,
    ));

    // Spawn coin counter
    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                format!("0/{coins}"),
                TextStyle {
                    font: asset_server.load("fonts/PixeloidSansBold.ttf"),
                    font_size: 40.,
                    color: TEXT_COLOR,
                },
            ),
            transform: Transform::from_translation(Vec3::new(
                (WIDTH / 2.) - 40.,
                (HEIGHT / 2.) - 40.,
                1.,
            )),
            text_anchor: Anchor::TopRight,
            ..default()
        },
        CoinCounter,
        OnGameScreen,
    ));

    // Spawn coin
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("images/coin.png"),
            sprite: Sprite {
                custom_size: Some(Vec2::ONE),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new((WIDTH / 2.) - 20., (HEIGHT / 2.) - 60., 0.),
                scale: Vec3::new(35., 35., 1.),
                ..default()
            },
            ..default()
        },
        OnGameScreen,
    ));

    // Spawn pause button
    commands.spawn((
        ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(BUTTON_SIZE), Val::Px(BUTTON_SIZE)),
                margin: UiRect::all(Val::Px(BUTTON_MARGIN)),
                position_type: PositionType::Absolute,
                ..default()
            },
            image: UiImage {
                texture: asset_server.load("images/pause.png"),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(-WIDTH / 2., HEIGHT / 2., 1.),
                ..default()
            },
            ..default()
        },
        OnGameScreen,
        ShowDuring::Running,
        ButtonAction::Pause,
    ));

    // Spawn resume button
    commands.spawn((
        ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(BUTTON_SIZE), Val::Px(BUTTON_SIZE)),
                margin: UiRect::all(Val::Px(BUTTON_MARGIN)),
                ..default()
            },
            image: UiImage {
                texture: asset_server.load("images/resume.png"),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(-WIDTH / 2., HEIGHT / 2., 1.),
                ..default()
            },
            visibility: Visibility::Hidden,
            ..default()
        },
        OnGameScreen,
        ShowDuring::Paused,
        ButtonAction::Resume,
    ));

    // Spawn quit button
    commands.spawn((
        ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(BUTTON_SIZE), Val::Px(BUTTON_SIZE)),
                margin: UiRect::all(Val::Px(BUTTON_MARGIN)),
                ..default()
            },
            image: UiImage {
                texture: asset_server.load("images/quit.png"),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(-WIDTH / 2., HEIGHT / 2., 1.),
                ..default()
            },
            visibility: Visibility::Hidden,
            ..default()
        },
        OnGameScreen,
        ShowDuring::Paused,
        ButtonAction::Quit,
    ));

    // Spawn pause text
    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                "Game Paused",
                TextStyle {
                    font: asset_server.load("fonts/PixeloidSansBold.ttf"),
                    font_size: 80.,
                    color: TEXT_COLOR,
                },
            ),
            visibility: Visibility::Hidden,
            transform: Transform::from_translation(Vec3::ZERO),
            ..default()
        },
        ShowDuring::Paused,
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
                    scale: Vec3::new(MAZE_BORDER_WIDTH, coord_size * 2. + MAZE_BORDER_WIDTH, 0.),
                    ..default()
                },
                ..default()
            },
            Collider,
            OnGameScreen,
            ShowDuring::Running,
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
                ShowDuring::Running,
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
    time: Res<Time>,
    maze_state: Res<MazeState>,
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

    let mul_const = time.delta_seconds() * PLAYER_SPEED;

    for transform in &collider_query {
        let collision_x = collide(
            transform.translation,
            transform.scale.truncate(),
            player_transform.translation + (Vec3::new(direction.x, 0., 0.) * mul_const),
            player_scale.truncate(),
        );

        let collision_y = collide(
            transform.translation,
            transform.scale.truncate(),
            player_transform.translation + (Vec3::new(0., direction.y, 0.) * mul_const),
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

    if !maze_state.stopwatch.paused() {
        player_transform.translation += direction * mul_const * player_scale;
    }
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

fn coin_count(
    mut coincounter_query: Query<&mut Text, With<CoinCounter>>,
    maze_state: ResMut<MazeState>,
    coins_query: Query<&Coin>,
) {
    let mut text = coincounter_query.single_mut();

    let coins = coins_query.iter().count();

    text.sections[0].value = format!("{}/{}", maze_state.coins - coins, maze_state.coins);
}

fn button_system(
    mut interaction_query: Query<
        (&Interaction, &ButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut collider_query: Query<(&mut Visibility, &ShowDuring), With<ShowDuring>>,
    mut maze_state: ResMut<MazeState>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for (interaction, button_action) in interaction_query.iter_mut() {
        if interaction == &Interaction::Clicked {
            match button_action {
                ButtonAction::Pause => {
                    maze_state.stopwatch.pause();
                    for (mut visibility, show_during) in collider_query.iter_mut() {
                        match *show_during {
                            ShowDuring::Paused => {
                                *visibility = Visibility::Visible;
                            }
                            ShowDuring::Running => {
                                *visibility = Visibility::Hidden;
                            }
                        }
                    }
                }
                ButtonAction::Resume => {
                    maze_state.stopwatch.unpause();
                    for (mut visibility, show_during) in collider_query.iter_mut() {
                        match *show_during {
                            ShowDuring::Paused => {
                                *visibility = Visibility::Hidden;
                            }
                            ShowDuring::Running => {
                                *visibility = Visibility::Visible;
                            }
                        }
                    }
                }
                ButtonAction::Quit => {
                    maze_state.stopwatch.pause();
                    game_state.set(GameState::Menu);
                }
            }
        }
    }
}
