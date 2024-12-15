mod common;
use common::MyRes;

use fecs::World;

#[test]
fn test_resources() {
    let mut world = World::default();

    // world.register_component::<MyRes>();
    // world.add_resource(MyRes(32));
}
