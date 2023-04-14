use crate::utils::{
    colors::{BACKGROUND_COLOR, HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON, TEXT_COLOR},
    despawn_screen,
    vars::{GameState, MazeState},
};
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
            maze_state.path = None;
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
                background_color: BACKGROUND_COLOR.into(),
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
                                background_color: NORMAL_BUTTON.into(),
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
