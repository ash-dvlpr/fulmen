#![allow(dead_code)]
use fulmen_ecs::*;

struct TestCompA(pub u32);
impl_component!(TestCompA, SparseSet);

struct TestCompB(pub u32);
impl_component!(TestCompB);

struct TestRes(pub u32);
impl_resource!(TestRes);


pub trait TestTrait { }
macro_rules! impl_test_trait {
    () => {};
    ($($name:ident),*) => {
        impl<$($name),*> TestTrait for ($($name,)*) { }
    };
}

// Generate tuples up to 8 elements (expand as needed)
gen_variadic_macro_calls!(impl_test_trait, D, C, B, A);