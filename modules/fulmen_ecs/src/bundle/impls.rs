// --- Imports
use crate::gen_variadic_macro_calls;
use crate::{bundle::Bundle, component::Component};

/// SAFETY:
/// - `Bundle::get_or_register_component_ids` only calls `id_callback` once per component.
/// - `Bundle::get_component_ids` only calls `id_callback` once per component.
unsafe impl<C: Component> Bundle for C {
    fn get_or_register_component_ids(
        world_components: &mut crate::component::Components,
        id_callback: &mut impl FnMut(crate::component::ComponentId),
    ) {
        id_callback(world_components.register_component::<C>());
    }

    fn get_component_ids(
        world_components: &crate::component::Components,
        id_callback: &mut impl FnMut(Option<crate::component::ComponentId>),
    ) {
        id_callback(world_components.component_id::<C>());
    }
}

macro_rules! bundle_tuple_impl {
    () => {};
    ($($name:ident),*) => {
        /// SAFETY:
        /// - `Bundle::get_or_register_component_ids` only calls `id_callback` once per component.
        /// - `Bundle::get_component_ids` only calls `id_callback` once per component.
        unsafe impl<$($name: Bundle),*> Bundle for ($($name,)*) {
            fn get_or_register_component_ids(
                world_components: &mut crate::component::Components,
                id_callback: &mut impl FnMut(crate::component::ComponentId),
            ) {
                $(<$name as Bundle>::get_or_register_component_ids(world_components, id_callback);)*
            }

            fn get_component_ids(
                world_components: &crate::component::Components,
                id_callback: &mut impl FnMut(Option<crate::component::ComponentId>),
            ) {
                $(<$name as Bundle>::get_component_ids(world_components, id_callback);)*
            }
        }
    };
}

// Generate tuples up to 16 elements (max)
gen_variadic_macro_calls!(bundle_tuple_impl, P, O, N, M, L, K, J, I, H, G, F, E, D, C, B, A);
