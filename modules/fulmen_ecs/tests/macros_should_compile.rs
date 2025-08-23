#![allow(dead_code)]
use fulmen_ecs::*;

#[derive(Component)]
struct TestCompA(pub u32);

#[derive(Component)]
struct TestCompB(pub u32);

#[derive(Resource)]
struct TestRes(pub u32);

pub trait TestTrait {}
macro_rules! impl_test_trait {
    () => {};
    ($($name:ident),*) => {
        impl<$($name),*> TestTrait for ($($name,)*) { }
    };
}

// Generate tuples up to 8 elements (expand as needed)
gen_variadic_macro_calls!(impl_test_trait, D, C, B, A);
