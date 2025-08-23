use fulmen_ecs::*;

#[derive(Default)]
pub struct InitRes {
    pub init_entities: Vec<Entity>,
}
fulmen_ecs::impl_resource!(InitRes);

pub struct CompA(pub usize);
fulmen_ecs::impl_component!(CompA);
pub struct CompB(pub usize);
fulmen_ecs::impl_component!(CompB);

// #[test]
// fn test_resources() {
//     // Create a new world, and initialize a required resource for the `init()` system
//     let mut world = World::new();
//     world.init_resource::<InitRes>();

//     // Create a schedule and add the `init()` system to it.
//     let mut init = Schedule::default();
//     init.add_systems((init_sys));

//     // Run the `Init` schedule, which will run the `init()` system.
//     init.run(&mut world);

//     // Create another schedule and add the `init()` system to it.
//     let mut post = Schedule::default();
//     post.add_systems((post_init_sys));

//     // Run the `Init` schedule, which will run the `init()` system.
//     post.run(&mut world);
// }

// pub fn init_sys(mut actions: Actions, init_res: MutRes<InitRes>) {
//     init_res
//         .init_entities
//         .push(actions.spawn(CompA(1), CompB(1)));
//     init_res
//         .init_entities
//         .push(actions.spawn(CompA(2), CompB(2)));
// }

// pub fn post_init_sys(init_res: Res<InitRes>) {
//     for handle in init_res.init_entities {

//     }
// }
