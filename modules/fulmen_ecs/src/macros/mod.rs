#[macro_export]
macro_rules! impl_resource {
    ( $t:ty ) => {
        impl fulmen_ecs::resource::Resource for $t {}
    };
}

#[macro_export]
macro_rules! impl_component {
    ( $t:ty ) => {
        impl_component!($t, fulmen_ecs::component::StorageType::SparseSet);
    };
    ( $t:ty, $storage_type:ident ) => {
        impl fulmen_ecs::component::Component for $t {
            const STORAGE_TYPE: fulmen_ecs::component::StorageType =
                fulmen_ecs::component::StorageType::$storage_type;
        }
    };
    ( $t:ty, $storage_type:expr ) => {
        impl fulmen_ecs::component::Component for $t {
            const STORAGE_TYPE: fulmen_ecs::component::StorageType = $storage_type;
        }
    };
}
