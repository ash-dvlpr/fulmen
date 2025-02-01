use fecs::*;

#[allow(dead_code)]
struct TestCompA(pub u32);
impl_component!(TestCompA, SparseSet);

#[allow(dead_code)]
struct TestCompB(pub u32);
impl_component!(TestCompB);

#[allow(dead_code)]
struct TestRes(pub u32);
impl_resource!(TestRes);
