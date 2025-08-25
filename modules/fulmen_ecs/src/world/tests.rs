#![allow(dead_code)]
use crate as fulmen_ecs;
use fulmen_ecs::*;

#[derive(Component)]
struct TestCompA(pub u32);

#[derive(Component)]
struct TestCompB(pub u32);

#[derive(Component)]
struct TestCompC(pub u32);

#[derive(Resource)]
struct TestResA(pub u32);

#[derive(Resource)]
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
