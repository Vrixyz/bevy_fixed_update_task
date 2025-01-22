//! This is a minimal example to show how synchronized fixed update works.

use bevy::dev_tools::fps_overlay::FpsOverlayPlugin;
use bevy::prelude::*;
use bevy_fixed_update_task::{
    BackgroundFixedUpdatePlugin, SpawnTaskSet, TaskResults, TaskToRenderTime, TaskWorker,
    TaskWorkerTrait, Timestep,
};
use bevy_rapier2d::prelude::*;

use std::{mem, time::Duration};

fn main() {
    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins,
        FpsOverlayPlugin::default(),
        BackgroundFixedUpdatePlugin::<TaskWorkerTraitImpl>::default(),
        RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0)
            .with_custom_initialization(RapierContextInitialization::NoAutomaticRapierContext)
            .in_schedule(bevy_fixed_update_task::FixedMain)
            .set_physics_sets_to_initialize([].into()),
        RapierDebugRenderPlugin::default(),
    ));
    app.add_systems(Startup, (setup_worker, (setup_info, setup_physics)).chain());
    app.add_systems(Update, update_info);
    // TODO: SyncBackend before [`SpawnTask`].
    app.add_systems(
        bevy_fixed_update_task::SpawnTask,
        RapierPhysicsPlugin::<NoUserData>::get_systems(PhysicsSet::SyncBackend)
            .in_set(SpawnTaskSet::PreSpawn),
    );
    // TODO: StepSimulation removed, that's our spawn task + handle task.
    // TODO: Writeback before [`PostWriteBack`].
    app.add_systems(
        bevy_fixed_update_task::PostWriteBack,
        RapierPhysicsPlugin::<NoUserData>::get_systems(PhysicsSet::Writeback),
    );

    // Run the app.
    app.run();
}

fn setup_worker(mut commands: Commands) {
    commands.spawn((
        Timestep {
            timestep: Duration::from_secs_f32(1.0 / 25.0),
        },
        TaskResults::<TaskWorkerTraitImpl>::default(),
        TaskWorker {
            worker: TaskWorkerTraitImpl {},
        },
        RapierContextSimulation::default(),
        DefaultRapierContext,
        RapierConfiguration {
            gravity: Vect::Y * -9.81 * 100.0,
            physics_pipeline_active: true,
            query_pipeline_active: true,
            scaled_shape_subdivision: 10,
            force_update_from_transform_changes: false,
        },
    ));
}

#[derive(Component)]
pub struct SimToRenderText;

pub fn setup_info(mut commands: Commands) {
    // Simulation to render time
    commands
        .spawn((
            Text::new("simulation to render time: "),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(50.0),
                left: Val::Px(15.0),
                ..default()
            },
        ))
        .with_child((TextSpan::default(), SimToRenderText));
}
pub fn update_info(
    task_to_render_time: Query<&TaskToRenderTime>,
    mut query: Query<&mut TextSpan, With<SimToRenderText>>,
) {
    for mut span in query.iter_mut() {
        **span = format!("{:.2}s", task_to_render_time.single().diff);
    }
}

pub fn setup_physics(mut commands: Commands) {
    let num = 80;
    let rad = 10.0;

    let shift = rad * 2.0 + rad;
    let centerx = shift * (num / 2) as f32;
    let centery = shift / 2.0;
    /*
     * Camera
     */
    commands.spawn((
        Camera2d::default(),
        OrthographicProjection {
            scale: 6f32,
            ..OrthographicProjection::default_2d()
        },
        Transform::from_xyz(-2500.0, 2080.0, 0.0),
    ));
    /*
     * Ground
     */
    let ground_size = 13500.0;
    let ground_height = 100.0;

    commands.spawn((
        Transform::from_xyz(-centerx, 0.0 * -ground_height - 100.0, 0.0),
        Collider::cuboid(ground_size, ground_height),
    ));

    /*
     * Create the cubes
     */
    let mut offset = -(num as f32) * (rad * 2.0 + rad) * 0.5;

    for j in 0usize..100 {
        for i in 0..num {
            let x = i as f32 * shift - centerx + offset;
            let y = j as f32 * shift + centery + 30.0;

            commands.spawn((
                // Mesh2d(mesh.clone()),
                //MeshMaterial2d(material.clone()),
                Transform::from_xyz(x, y, 0.0),
                RigidBody::Dynamic,
                Collider::cuboid(rad, rad),
            ));
        }

        offset -= 0.05 * rad * ((num as f32 * 1.0) - 1.0);
    }
}

#[derive(Debug, Clone, Default)]
pub struct TaskWorkerTraitImpl;

impl TaskWorkerTrait for TaskWorkerTraitImpl {
    type TaskExtractedData = TaskExtractedData;
    type TaskResultPure = TaskResult;

    fn work(
        &self,
        _worker: Entity,
        mut input: TaskExtractedData,
        timestep: Duration,
        substep_count: u32,
    ) -> Self::TaskResultPure {
        input.rapier_context.step_simulation(
            &mut input.colliders,
            &mut input.joints,
            &mut input.bodies,
            input.configuration.gravity,
            TimestepMode::Fixed {
                dt: timestep.as_secs_f32(),
                substeps: substep_count as usize,
            },
            None, // FIXME: change `None` to `true` (see bevy's integration from Thierry)
            &(),  // FIXME: &hooks_adapter,
            &input.time,
            &mut input.sim_to_render_time,
            None,
        );
        TaskResult {
            rapier_context: input.rapier_context,
            colliders: input.colliders,
            bodies: input.bodies,
            joints: input.joints,
            query_pipeline: input.query_pipeline,
            sim_to_render_time: input.sim_to_render_time,
        }
    }

    fn extract(&self, worker_entity: Entity, world: &mut World) -> TaskExtractedData {
        // Time is not actually used as we're only using `TimestepMode::Fixed`,
        // but rapier API requires it.
        let time = world.get_resource::<Time>().unwrap();

        let time = time.clone();
        let mut rapier_context_query = world.query::<(
            &mut RapierContextSimulation,
            &RapierContextColliders,
            &RapierRigidBodySet,
            &RapierContextJoints,
            &RapierQueryPipeline,
            &RapierConfiguration,
            &mut SimulationToRenderTime,
        )>();
        let (
            mut context_ecs,
            colliders,
            bodies,
            joints,
            query_pipeline,
            config,
            sim_to_render_time,
        ) = rapier_context_query.get_mut(world, worker_entity).unwrap();

        // FIXME: Clone this properly?
        let mut rapier_context = RapierContextSimulation::default();
        mem::swap(&mut rapier_context, &mut *context_ecs);
        // TODO: use a double buffering system to avoid this more expensive (to verify) cloning.
        let colliders = colliders.clone();
        let bodies = bodies.clone();
        let joints = joints.clone();
        let query_pipeline = query_pipeline.clone();
        // let mut context: RapierContext =
        //    unsafe { mem::transmute_copy::<RapierContext, RapierContext>(&*context_ecs) };
        let configuration = config.clone();

        let sim_to_render_time = sim_to_render_time.clone();

        TaskExtractedData {
            time,
            rapier_context,
            colliders,
            bodies,
            joints,
            query_pipeline,
            configuration,
            sim_to_render_time,
        }
    }

    fn write_back(
        &self,
        worker_entity: Entity,
        result: bevy_fixed_update_task::TaskResult<Self>,
        world: &mut World,
    ) {
        let mut rapier_context_query = world.query::<(
            &mut RapierContextSimulation,
            &mut RapierContextColliders,
            &mut RapierRigidBodySet,
            &mut RapierContextJoints,
            &mut RapierQueryPipeline,
            &mut SimulationToRenderTime,
        )>();
        let (
            mut context_ecs,
            mut colliders,
            mut bodies,
            mut joints,
            mut query_pipeline,
            mut sim_to_render_time,
        ) = rapier_context_query.get_mut(world, worker_entity).unwrap();

        *context_ecs = result.result_raw.result.rapier_context;
        *colliders = result.result_raw.result.colliders;
        *bodies = result.result_raw.result.bodies;
        *joints = result.result_raw.result.joints;
        *query_pipeline = result.result_raw.result.query_pipeline;
        *sim_to_render_time = result.result_raw.result.sim_to_render_time;
    }
}

#[derive(Component)]
pub struct TaskExtractedData {
    pub time: Time,
    pub rapier_context: RapierContextSimulation,
    pub colliders: RapierContextColliders,
    pub bodies: RapierRigidBodySet,
    pub joints: RapierContextJoints,
    pub query_pipeline: RapierQueryPipeline,
    pub configuration: RapierConfiguration,
    pub sim_to_render_time: SimulationToRenderTime,
}

#[derive(Component)]
pub struct TaskResult {
    pub rapier_context: RapierContextSimulation,
    pub colliders: RapierContextColliders,
    pub bodies: RapierRigidBodySet,
    pub joints: RapierContextJoints,
    pub query_pipeline: RapierQueryPipeline,
    pub sim_to_render_time: SimulationToRenderTime,
}
