use fulmen_ecs::*;

#[allow(dead_code)]
pub struct MyRes(pub usize);
fulmen_ecs::impl_resource!(MyRes);

#[test]
fn test_resources() {
    let mut world = World::default();

    world.insert_resource(MyRes(32));
}
