pub trait Resource: Send + Sync + 'static {}

#[macro_export]
macro_rules! impl_resource {
    ( $( $t:ty ),* ) => {
        $(
            impl fulmen_ecs::resource::Resource for $t {}
        )*
    };
}
