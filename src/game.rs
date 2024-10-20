use bevy::prelude::*;
use rand::prelude::*;

use super::{despawn_screen, DisplayQuality, GameState, Volume, TEXT_COLOR};
pub fn game_plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Game), game_setup)
        .add_systems(OnExit(GameState::Game), despawn_screen::<OnGameScreen>)
        .add_systems(Update, (snake_direction_change).chain())
        .add_systems(FixedUpdate, (snake_movement, size_scaling, position_translation).chain())
        .observe(body_spawner)
        .observe(food_spawner)
        .observe(body_follow_front)
        .observe(check_snake_eat_food)
        .observe(check_snake_eat_body)
        .observe(game_over);
}

const ARENA_WIDTH: i32 = 10;
const ARENA_HEIGHT: i32 = 10;

#[derive(Debug)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Component)]
struct SnakeHead {
    direction: Direction,
    bodies: Vec<Entity>,
}

#[derive(Deref, DerefMut)]
struct SnakeMoveTimer(Timer);
impl Default for SnakeMoveTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(0.8, TimerMode::Repeating))
    }
}

#[derive(Component)]
struct OnGameScreen;

#[derive(Component)]
struct Food;

#[derive(Component)]
struct SnakeBody;

#[derive(Component)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Component)]
struct Size {
    scale: f32,
}
#[derive(Component)]
struct Collider;
#[derive(Event)]
struct SpawnBody {
    body_position: Position,
}
#[derive(Event)]
struct SpawnFood;

#[derive(Event)]
struct FollowBody {
    snake_position: Position,
}

#[derive(Event)]
struct CheckSnakeEatFood {
    snake_position: Position,
}

#[derive(Event)]
struct CheckSnakeEatBody {
    snake_position: Position,
}

#[derive(Event)]
struct GameOverEvent;

fn game_setup(mut commands: Commands) {
    // camera
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    // center children
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
            OnGameScreen,
        ));
    spawn_snake(&mut commands);
    commands.trigger(SpawnFood);
}

fn spawn_snake(commands: &mut Commands) {
    let default_position_x = ARENA_WIDTH / 2;
    let default_position_y = ARENA_HEIGHT / 2;

    let default_direction = Direction::Up;
    let (first_body_position_x, first_body_position_y) = match default_direction {
        Direction::Left => {
            (default_position_x + 1, default_position_y)
        },
        Direction::Right => {
            (default_position_x - 1, default_position_y)
        },
        Direction::Down => {
            (default_position_x, default_position_y + 1)
        },
        Direction::Up => {
            (default_position_x, default_position_y - 1)
        },
    };
    // snake head
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgba(0.7, 0.7, 0.7, 1.0),
                ..default()
            },
            ..default()
        },
        SnakeHead {
            direction: default_direction,
            bodies: Vec::new(),
        },
        Position {
            x: default_position_x,
            y: default_position_y,
        },
        Size {
            scale: 0.8
        },
        Collider,
    ));

    commands.trigger(SpawnBody {
        body_position: Position {
            x: first_body_position_x,
            y: first_body_position_y,
        },
    });
}

fn size_scaling(mut windows: Query<&mut Window>, mut query: Query<(&Size, &mut Sprite)>) {
    for window in windows.iter_mut() {
        for (size, mut sprite) in &mut query.iter_mut() {
            sprite.custom_size = Some(Vec2::new(window.width() / ARENA_WIDTH as f32 * size.scale, window.height() / ARENA_HEIGHT as f32 * size.scale));
            if let Some(mut custom_size) = sprite.custom_size {
                let width = window.width() / ARENA_WIDTH as f32 * size.scale;
                let height = window.height() / ARENA_HEIGHT as f32 * size.scale;
                if width != custom_size.x {
                    custom_size.x = width;
                }
                if height != custom_size.y {
                    custom_size.y = height;
                }
            } else {
                sprite.custom_size = Some(Vec2::new(window.width() / ARENA_WIDTH as f32 * size.scale, window.height() / ARENA_HEIGHT as f32 * size.scale));
            }
        }
    }
}

fn position_translation(mut windows: Query<&mut Window>, mut query: Query<(&Position, &mut Transform)>) {
    fn convert(pos: f32, bound_window: f32, bound_game: f32) -> f32 {
        pos * (bound_window / bound_game) - (bound_window / 2.0) + (bound_window / bound_game / 2.0)
    }
    for window in windows.iter_mut() {
        for (pos, mut transform) in &mut query.iter_mut() {
            let translation_x = convert(pos.x as f32, window.width(), ARENA_WIDTH as f32);
            let translation_y = convert(pos.y as f32, window.height(), ARENA_HEIGHT as f32);
            if translation_x != transform.translation.x {
                transform.translation.x = translation_x;
            }
            if translation_y != transform.translation.y {
                transform.translation.y = translation_y;
            }
        }
    }
}

fn snake_direction_change(keyboard_input: Res<ButtonInput<KeyCode>>, mut query: Query<&mut SnakeHead>) {
    if let Some(mut head) = query.iter_mut().next() {
        if keyboard_input.pressed(KeyCode::ArrowLeft) {
            if !matches!(head.direction, Direction::Right) {
                head.direction = Direction::Left;
            }
        } else if keyboard_input.pressed(KeyCode::ArrowRight) {
            if !matches!(head.direction, Direction::Left) {
                head.direction = Direction::Right;
            }
        } else if keyboard_input.pressed(KeyCode::ArrowDown) {
            if !matches!(head.direction, Direction::Up) {
                head.direction = Direction::Down;
            }
        } else if keyboard_input.pressed(KeyCode::ArrowUp) {
            if !matches!(head.direction, Direction::Down) {
                head.direction = Direction::Up;
            }
        }
    }
}

fn snake_movement(mut commands: Commands, mut query_snake_head: Query<(&mut SnakeHead, &mut Position)>, mut timer: Local<SnakeMoveTimer>, time: Res<Time>,) {
    timer.tick(time.delta());
    if timer.finished() {
        for (snake_head, mut position_snake_head) in query_snake_head.iter_mut() {
            let mut next_translation_x = position_snake_head.x;
            let mut next_translation_y = position_snake_head.y;
            match snake_head.direction {
                Direction::Left => {
                    next_translation_x -= 1;
                }
                Direction::Right => {
                    next_translation_x += 1;
                }
                Direction::Down => {
                    next_translation_y -= 1;
                }
                Direction::Up => {
                    next_translation_y += 1;
                }
            }

            if next_translation_x < 0 || next_translation_x >= ARENA_WIDTH || next_translation_y < 0 || next_translation_y >= ARENA_HEIGHT {
                commands.trigger(GameOverEvent);
            } else {
                commands.trigger(CheckSnakeEatBody {
                    snake_position: Position {
                        x: next_translation_x,
                        y: next_translation_y
                    },
                });

                commands.trigger(CheckSnakeEatFood {
                    snake_position: Position {
                        x: next_translation_x,
                        y: next_translation_y
                    },
                });

                commands.trigger(FollowBody {
                    snake_position: Position {
                        x: position_snake_head.x,
                        y: position_snake_head.y
                    },
                });

                position_snake_head.x = next_translation_x;
                position_snake_head.y = next_translation_y;
            }
        }
    }
}

fn body_spawner(trigger: Trigger<SpawnBody>, mut commands: Commands, mut query_snake_head: Query<&mut SnakeHead>) {
    let event = trigger.event();
    let mut snake_head = query_snake_head.single_mut();
    snake_head.bodies.push(commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgba(0.3, 0.3, 0.3, 1.0),
                ..default()
            },
            ..default()
        },
        SnakeBody,
        Position {
            x: event.body_position.x,
            y: event.body_position.y,
        },
        Size {
            scale: 0.65
        },
        Collider,
    )).id());
}

fn food_spawner(trigger: Trigger<SpawnFood>, mut commands: Commands, mut query_position: Query<&Position>) {
    let _event = trigger.event();
    let mut numbers: Vec<i32> = (0..ARENA_WIDTH * ARENA_HEIGHT).collect();
    for position in &mut query_position.iter_mut() {
        numbers.retain(|&x| x != position.x + position.y * ARENA_HEIGHT);
    }
    let mut rng = thread_rng();
    if numbers.len() > 0 {
        let rand = rng.gen_range(0..numbers.len());
        let rand_number = numbers.get(rand);
        if let Some(rand_number) = rand_number {
            let rand_position_x = rand_number % ARENA_WIDTH;
            let rand_position_y = rand_number / ARENA_HEIGHT;
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgba(1.0, 0.0, 1.0, 1.0),
                        ..default()
                    },
                    ..default()
                },
                Food,
                Position {
                    x: rand_position_x,
                    y: rand_position_y,
                },
                Size {
                    scale: 0.8
                },
                Collider,
            ));
        }
    } else {
        commands.trigger(GameOverEvent);
    }
}

fn body_follow_front(trigger: Trigger<FollowBody>, mut query_snake_head: Query<&mut SnakeHead>, mut query_snake_position: Query<&mut Position>) {
    let event = trigger.event();
    for snake_head in query_snake_head.iter_mut() {
        let mut iter_body = snake_head.bodies.iter().rev().peekable();
        while let Some(entity_now_body) = iter_body.next() {
            let (next_position_x, next_position_y) = if let Some(entity_previous_body) = iter_body.peek() {
                if let Ok(position_previous_body) = query_snake_position.get(**entity_previous_body) {
                    (Some(position_previous_body.x), Some(position_previous_body.y))
                } else {
                    (None, None)
                }
            } else {
                (Some(event.snake_position.x), Some(event.snake_position.y))
            };

            if let Ok(mut position_now_body) = query_snake_position.get_mut(*entity_now_body) {
                if let (Some(next_position_x_val), Some(next_position_y_val)) = (next_position_x, next_position_y) {
                    position_now_body.x = next_position_x_val;
                    position_now_body.y = next_position_y_val;
                }
            }
        }
    }
}

fn check_snake_eat_food(trigger: Trigger<CheckSnakeEatFood>, mut commands: Commands, mut query_food: Query<(&mut Position, Entity), With<Food>>) {
    let event = trigger.event();
    for (position, entity) in query_food.iter_mut() {
        if event.snake_position.x == position.x && event.snake_position.y == position.y {
            commands.entity(entity).despawn();
            commands.trigger(SpawnBody {
                body_position: Position {
                    x: position.x,
                    y: position.y,
                },
            });
            commands.trigger(SpawnFood);
        }
    }
}

fn check_snake_eat_body(trigger: Trigger<CheckSnakeEatBody>, mut commands: Commands, mut query_snake_body: Query<&Position, With<SnakeBody>>) {
    let event = trigger.event();
    for position in query_snake_body.iter_mut() {
        if event.snake_position.x == position.x && event.snake_position.y == position.y {
            commands.trigger(GameOverEvent);
        }
    }
}

fn game_over(trigger: Trigger<GameOverEvent>, mut game_state: ResMut<NextState<GameState>>, mut commands: Commands, query_snake_head: Query<Entity, With<SnakeHead>>, query_snake_body: Query<Entity, With<SnakeBody>>, query_food: Query<Entity, With<Food>>) {
    let _event = trigger.event();
    for entity in query_snake_head.iter() {
        commands.entity(entity).despawn();
    }
    for entity in query_snake_body.iter() {
        commands.entity(entity).despawn();
    }
    for entity in query_food.iter() {
        commands.entity(entity).despawn();
    }

    game_state.set(GameState::Menu);
}