use std::{cell::RefCell, rc::Rc};

use bevy::{
    core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping},
    prelude::*,
    sprite::MaterialMesh2dBundle,
    window::PrimaryWindow,
};
use random_color::{Luminosity, RandomColor};

use crate::model::Universe;

const POLYGON_DIMENSION: f32 = 12.5;
const POLYGON_SIDES: usize = 4;
const POLYGON_ROTATION_DEGREES: f32 = 45.0;

const BUTTON_UP: Color = Color::rgb(0.15, 0.15, 0.15);
const BUTTON_DOWN: Color = Color::rgb(0.35, 0.75, 0.35);

#[derive(Resource)]
struct UniverseTickTimer(Timer);

#[derive(Debug, Clone, Copy, Component)]
pub struct Cell {
    pub x: usize,
    pub y: usize,
    pub alive: bool,
    pub base_color: [u8; 3],
    pub heat: f32,
    pub last_changed: f64,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Query<&Window>,
    wrapped_universe: NonSend<Rc<RefCell<Universe>>>,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            tonemapping: Tonemapping::TonyMcMapface,
            ..default()
        },
        BloomSettings::default(),
    ));

    let window = window.single();
    let width = window.resolution.width();
    let height = window.resolution.height();
    let cols = ((width / (POLYGON_DIMENSION * 2.0)) as usize) - 3;
    let rows = ((height / (POLYGON_DIMENSION * 2.0)) as usize) - 2;

    let mut universe = wrapped_universe.as_ref().borrow_mut();
    *universe = Universe::from_dimensions(cols, rows).unwrap();

    for i in 0..rows {
        for j in 0..cols {
            let x =
                (j as f32 * POLYGON_DIMENSION * 2.0) - (width / 2.0) + (POLYGON_DIMENSION) + (POLYGON_DIMENSION * 3.0);
            let y =
                (i as f32 * POLYGON_DIMENSION * 2.0) - (height / 2.0) + (POLYGON_DIMENSION) + (POLYGON_DIMENSION * 3.0);

            let material_mesh_2d_bundle = MaterialMesh2dBundle {
                mesh: bevy::sprite::Mesh2dHandle(meshes.add(
                    <bevy::prelude::RegularPolygon as std::convert::Into<Mesh>>::into(
                        bevy::math::prelude::RegularPolygon::new(POLYGON_DIMENSION, POLYGON_SIDES),
                    ),
                )),
                material: materials.add(ColorMaterial::from(Color::DARK_GRAY)),
                transform: Transform::from_translation(Vec3::new(x, y, 0.0))
                    .with_rotation(Quat::from_rotation_z(POLYGON_ROTATION_DEGREES.to_radians())),
                ..default()
            };

            let cell = Cell {
                x: j,
                y: i,
                base_color: RandomColor::new()
                    .luminosity(Luminosity::Bright)
                    .to_rgb_array(),
                alive: false,
                heat: 1.0,
                last_changed: time.elapsed_seconds_f64() - 1.0,
            };

            let _parent = commands.spawn((material_mesh_2d_bundle, cell));
        }
    }

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Relative,
                top: Val::Percent(96.0),
                left: Val::Percent(0.5),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(100.0),
                        height: Val::Px(25.0),
                        border: UiRect::all(Val::Px(1.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    border_color: BorderColor(Color::WHITE),
                    background_color: Color::rgb(0.15, 0.15, 0.15).into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Play / Pause",
                        TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 16.0,
                            color: Color::WHITE,
                        },
                    ));
                });
        });
}

fn handle_cell_sync(
    mut q_materials_for_cells: Query<(&Handle<ColorMaterial>, &Cell), With<Cell>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (material_handle, cell) in q_materials_for_cells.iter_mut() {
        let possible_material = materials.get_mut(material_handle.id());
        if possible_material.is_none() {
            continue;
        }

        let material = possible_material.unwrap();

        if cell.alive {
            material.color = Color::rgb(
                (cell.base_color[0] as f32) / 255.0 * cell.heat,
                (cell.base_color[1] as f32) / 255.0 * cell.heat,
                (cell.base_color[2] as f32) / 255.0 * cell.heat,
            );
        } else {
            material.color = Color::DARK_GRAY;
        }
    }
}

fn handle_universe_sync(
    mut q_cells: Query<&mut Cell>,
    wrapped_universe: NonSend<Rc<RefCell<Universe>>>,
) {
    let universe = wrapped_universe.as_ref().borrow();

    let rows = universe.rows();

    for row in rows.iter() {
        for universe_cell in row.iter() {
            let (x, y) = universe_cell.coords();

            for mut cell in q_cells.iter_mut() {
                if (cell.x, cell.y) != (x, y) {
                    continue;
                }

                cell.alive = universe_cell.alive();
                cell.heat = universe_cell.heat();
            }
        }
    }
}

fn handle_universe_tick(
    time: Res<Time>,
    mut timer: ResMut<UniverseTickTimer>,
    wrapped_universe: NonSend<Rc<RefCell<Universe>>>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    let mut universe = wrapped_universe.as_ref().borrow_mut();

    universe.tick();
}

fn handle_clicks(
    q_windows: Query<&Window, With<PrimaryWindow>>,
    mut q_transforms_for_cells: Query<(&Transform, &mut Cell), With<Cell>>,
    buttons: Res<ButtonInput<MouseButton>>,
    wrapped_universe: NonSend<Rc<RefCell<Universe>>>,
    time: Res<Time>,
) {
    if !buttons.pressed(MouseButton::Left) {
        return;
    };

    let window = q_windows.single();
    let width = window.resolution.width();
    let height = window.resolution.height();

    if window.cursor_position().is_none() {
        return;
    }

    let position = window.cursor_position().unwrap();
    let cursor = Vec3::new(position.x - (width / 2.0), (height / 2.0) - position.y, 0.0);

    let mut universe = wrapped_universe.as_ref().borrow_mut();

    for (transform, mut cell) in q_transforms_for_cells.iter_mut() {
        if transform.translation.distance(cursor) > POLYGON_DIMENSION {
            continue;
        }

        if time.elapsed_seconds_f64() - cell.last_changed < 1.0 {
            return;
        }

        cell.alive = !cell.alive;
        cell.last_changed = time.elapsed_seconds_f64();
        universe.set_alive((cell.x, cell.y), cell.alive, 0);
    }
}

fn handle_touches(
    q_windows: Query<&Window, With<PrimaryWindow>>,
    mut q_transforms_for_cells: Query<(&Transform, &mut Cell), With<Cell>>,
    touches: Res<Touches>,
    wrapped_universe: NonSend<Rc<RefCell<Universe>>>,
    time: Res<Time>,
) {
    let window = q_windows.single();
    let width = window.resolution.width();
    let height = window.resolution.height();

    let mut universe = wrapped_universe.as_ref().borrow_mut();

    for touch in touches.iter() {
        if touches.get_pressed(touch.id()).is_none() {
            continue;
        }

        let position = touch.position();
        let cursor = Vec3::new(position.x - (width / 2.0), (height / 2.0) - position.y, 0.0);

        for (transform, mut cell) in q_transforms_for_cells.iter_mut() {
            if transform.translation.distance(cursor) > POLYGON_DIMENSION {
                continue;
            }

            if time.elapsed_seconds_f64() - cell.last_changed < 1.0 {
                return;
            }

            cell.alive = !cell.alive;
            cell.last_changed = time.elapsed_seconds_f64();
            universe.set_alive((cell.x, cell.y), cell.alive, 0);
        }
    }
}

fn handle_keys(keys: Res<ButtonInput<KeyCode>>, wrapped_universe: NonSend<Rc<RefCell<Universe>>>) {
    let mut universe = wrapped_universe.as_ref().borrow_mut();

    if keys.just_pressed(KeyCode::KeyR) {
        universe.reset();
    }

    if keys.just_pressed(KeyCode::Space) {
        universe.pause();
    }
}

#[allow(clippy::type_complexity)]
fn handle_buttons(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    wrapped_universe: NonSend<Rc<RefCell<Universe>>>,
) {
    let mut universe = wrapped_universe.as_ref().borrow_mut();

    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = BUTTON_DOWN.into();
                universe.pause();
            }
            Interaction::Hovered => {}
            Interaction::None => {
                *color = BUTTON_UP.into();
            }
        }
    }
}

pub fn run() {
    let universe = Universe::default();

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                canvas: Some("#game-of-life-canvas".into()),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, handle_universe_sync)
        .add_systems(Update, handle_cell_sync)
        .add_systems(Update, handle_universe_tick)
        .add_systems(Update, handle_clicks)
        .add_systems(Update, handle_keys)
        .add_systems(Update, handle_touches)
        .add_systems(Update, handle_buttons)
        .insert_resource(ClearColor(Color::rgb(0.125, 0.125, 0.125)))
        .insert_non_send_resource(Rc::new(RefCell::new(universe)))
        .insert_resource(UniverseTickTimer(Timer::from_seconds(
            0.001,
            TimerMode::Repeating,
        )))
        .run();
}
