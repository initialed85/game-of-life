use std::{cell::RefCell, rc::Rc};

use bevy::{
    core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping},
    prelude::*,
    sprite::MaterialMesh2dBundle,
    window::PrimaryWindow,
};
use random_color::{Luminosity, RandomColor};

use crate::model::Universe;

const POLYGON_DIMENSION: f32 = 6.25 * 2.0;
const POLYGON_SIDES: usize = 4;
const POLYGON_ROTATION_DEGREES: f32 = 45.0;

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
    let cols = (width / (POLYGON_DIMENSION * 2.0)) as usize;
    let rows = (height / (POLYGON_DIMENSION * 2.0)) as usize;

    let mut universe = wrapped_universe.as_ref().borrow_mut();
    *universe = Universe::from_dimensions(cols, rows).unwrap();

    for i in 0..rows {
        for j in 0..cols {
            let x =
                (j as f32 * POLYGON_DIMENSION * 2.0) - (width / 2.0) + (POLYGON_DIMENSION * 1.25);
            let y =
                (i as f32 * POLYGON_DIMENSION * 2.0) - (height / 2.0) + (POLYGON_DIMENSION * 1.75);

            let material_mesh_2d_bundle = MaterialMesh2dBundle {
                mesh: meshes
                    .add(shape::RegularPolygon::new(POLYGON_DIMENSION, POLYGON_SIDES).into())
                    .into(),
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

fn handle_cell_click(
    q_windows: Query<&Window, With<PrimaryWindow>>,
    mut q_transforms_for_cells: Query<(&Transform, &mut Cell), With<Cell>>,
    buttons: Res<Input<MouseButton>>,
    wrapped_universe: NonSend<Rc<RefCell<Universe>>>,
    time: Res<Time>,
) {
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

        if !buttons.pressed(MouseButton::Left) {
            return;
        };

        if time.elapsed_seconds_f64() - cell.last_changed < 1.0 {
            return;
        }

        cell.alive = !cell.alive;
        cell.last_changed = time.elapsed_seconds_f64();
        universe.set_alive((cell.x, cell.y), cell.alive, 0);
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

fn handle_reset(keys: Res<Input<KeyCode>>, wrapped_universe: NonSend<Rc<RefCell<Universe>>>) {
    let mut universe = wrapped_universe.as_ref().borrow_mut();

    if keys.just_pressed(KeyCode::R) {
        universe.reset();
    }

    if keys.just_pressed(KeyCode::Space) {
        universe.pause();
    }
}

pub fn run() {
    let universe = Universe::default();

    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, handle_universe_sync)
        .add_systems(Update, handle_cell_click)
        .add_systems(Update, handle_cell_sync)
        .add_systems(Update, handle_universe_tick)
        .add_systems(Update, handle_reset)
        .insert_resource(ClearColor(Color::rgb(0.125, 0.125, 0.125)))
        .insert_non_send_resource(Rc::new(RefCell::new(universe)))
        .insert_resource(UniverseTickTimer(Timer::from_seconds(
            0.0000001,
            TimerMode::Repeating,
        )))
        .run();
}
