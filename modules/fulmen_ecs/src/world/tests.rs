#![allow(dead_code)]
use crate as fulmen_ecs;
use fulmen_ecs::*;

#[derive(Component, Default)]
struct TestCompA(pub u32);

#[derive(Component, Default)]
struct TestCompB(pub u32);

#[derive(Component, Default)]
struct TestCompC(pub u32);

#[derive(Resource, Default)]
struct TestResA(pub u32);

#[derive(Resource, Default)]
struct TestResB(pub u32);

#[test]
fn register_components() {
    let mut world = World::new();

    // New components
    assert_eq!(0, world.register_component::<TestCompA>().index());
    assert_eq!(1, world.register_component::<TestCompB>().index());
    assert_eq!(2, world.register_component::<TestCompC>().index());

    // Already registered component
    assert_eq!(0, world.register_component::<TestCompA>().index());
}

#[test]
fn register_resources() {
    let mut world = World::new();

    // New Resources
    assert_eq!(0, world.register_resource::<TestResA>().index());
    assert_eq!(1, world.register_resource::<TestResB>().index());

    // Already registered resource
    assert_eq!(0, world.register_resource::<TestResA>().index());
}

#[test]
fn register_components_and_resources() {
    let mut world = World::new();

    // New components
    assert_eq!(0, world.register_component::<TestCompA>().index());

    // New Resources
    assert_eq!(1, world.register_resource::<TestResA>().index());

    // New components
    assert_eq!(2, world.register_component::<TestCompB>().index());

    // New Resources
    assert_eq!(3, world.register_resource::<TestResB>().index());

    // Already registered resource
    assert_eq!(1, world.register_resource::<TestResA>().index());
}

#[test]
fn recover_registered_resources() {
    let mut world = World::new();

    // Get unregistered
    let res = world.get_resource::<TestResA>();
    assert!(res.is_none(), "res isn't registered");

    // Resources registration after checking existance
    assert_eq!(
        0,
        world.register_resource::<TestResA>().index(),
        "registered resource should have id 0 as 'get_resource' should not register unregistered types"
    );

    // Get registered + uninit
    let res = world.get_resource::<TestResA>();
    assert!(res.is_none(), "res shouldn't be initialized");

    // Get initialized
    world.insert_resource(TestResA(6));
    let og_res = world.get_resource::<TestResA>();
    assert!(og_res.is_some(), "res should be initialized");
    assert_eq!(6, og_res.unwrap().0);

    // Check init doesn't override
    world.init_resource::<TestResA>();
    let res = world.get_resource::<TestResA>();
    assert_eq!(6, res.unwrap().0);

    // Check insert overrides
    world.insert_resource(TestResA(1));
    let res = world.get_resource::<TestResA>();
    assert_eq!(1, res.unwrap().0);
}

#[test]
#[should_panic(expected = "acquire uninitialized resource")]
fn recover_unregistered_resource() {
    let world = World::new();

    // Extract unregistered resource
    _ = world.resource::<TestResA>();
}

#[test]
fn resource_mutation() {
    let mut world = World::new();

    world.init_resource::<TestResA>();

    // Default value
    let res = world.resource_mut::<TestResA>();
    assert_eq!(TestCompA::default().0, res.0);

    res.0 = 10;
    assert_eq!(10, res.0);

    // New borrow
    let res = world.resource_mut::<TestResA>();
    assert_eq!(10, res.0);
}

#[test]
fn register_bundle_of_registered_components() {
    let mut world = World::new();

    // Register components
    _ = world.register_component::<TestCompA>();
    _ = world.register_component::<TestCompB>();
    _ = world.register_component::<TestCompC>();

    let World {
        components,
        bundles,
        ..
    } = &mut world;

    // Already registered components
    _ = bundles.register_bundle::<TestCompA>(components);
    _ = bundles.register_bundle::<TestCompB>(components);
    _ = bundles.register_bundle::<TestCompC>(components);
    _ = bundles.register_bundle::<(TestCompA, TestCompB)>(components);
    _ = bundles.register_bundle::<(TestCompB, TestCompC)>(components);
    _ = bundles.register_bundle::<(TestCompC, TestCompA)>(components);
    _ = bundles.register_bundle::<(TestCompA, (TestCompB, TestCompC))>(components);

    assert_eq!(3, components.len());
    assert_eq!(7, bundles.len());
}

#[test]
fn register_bundle_of_unregistered_components() {
    let mut world = World::new();
    let World {
        components,
        bundles,
        ..
    } = &mut world;

    // Unregistered components
    _ = bundles.register_bundle::<TestCompA>(components);
    _ = bundles.register_bundle::<TestCompB>(components);
    _ = bundles.register_bundle::<TestCompC>(components);
    _ = bundles.register_bundle::<(TestCompA, TestCompB)>(components);
    _ = bundles.register_bundle::<(TestCompB, TestCompC)>(components);
    _ = bundles.register_bundle::<(TestCompC, TestCompA)>(components);
    _ = bundles.register_bundle::<(TestCompA, (TestCompB, TestCompC))>(components);

    assert_eq!(3, components.len());
    assert_eq!(7, bundles.len());
}

#[test]
#[should_panic(expected = "contains duplicated Components")]
fn register_bundle_duplicated_components() {
    let mut world = World::new();
    let World {
        components,
        bundles,
        ..
    } = &mut world;

    _ = bundles.register_bundle::<(TestCompA, (TestCompB, TestCompA))>(components);
}
