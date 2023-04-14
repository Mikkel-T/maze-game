use crate::utils::{
    colors::{BACKGROUND_COLOR, HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON, TEXT_COLOR},
    despawn_screen,
    vars::{GameState, MazeState},
};
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

#[derive(Component)]
struct ButtonAction(GameState);

fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &ButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut game_state: ResMut<NextState<GameState>>,
    mut maze_state: ResMut<MazeState>,
) {
    for (interaction, mut color, state) in &mut interaction_query {
        *color = match *interaction {
            Interaction::Clicked => PRESSED_BUTTON.into(),
            Interaction::Hovered => HOVERED_BUTTON.into(),
            Interaction::None => NORMAL_BUTTON.into(),
        };

        if *interaction == Interaction::Clicked {
            maze_state.stopwatch.reset();
            maze_state.stopwatch.unpause();
            game_state.set(state.0);
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
                background_color: BACKGROUND_COLOR.into(),
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
                            background_color: NORMAL_BUTTON.into(),
                            ..default()
                        })
                        .insert(ButtonAction(GameState::Menu))
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
                            background_color: NORMAL_BUTTON.into(),
                            ..default()
                        })
                        .insert(ButtonAction(GameState::Game))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                "Try again",
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
