/// Reverses the parameters passed to the before calling m!()
#[macro_export]
macro_rules! reverse_and_call {
    // Base case with no [pending] elements, calls `m`.
    ($m:ident [] $($reversed:tt)*) => {
        $m!{$($reversed),*}
    };
    ($m:ident [$first:tt $($rest:tt)*] $($reversed:tt)*) => {
        crate::reverse_and_call!{$m [$($rest)*] $first $($reversed)*}
    };
}

// Generates variadic macro calls for a macro.
//
// Calls m!() with up to the specified number of params
// to generate code for fake variadic implementations.
#[macro_export]
macro_rules! gen_variadic_macro_calls {
    // Base case for the recursive macro
    ($m: ident, $param: tt) => {
        $m!{$param}
    };
    // 
    ($m: ident, $param: tt, $($rest_params: tt),*) => {
        crate::gen_variadic_macro_calls!{$m, $($rest_params),*}
        crate::reverse_and_call!{$m [$param $($rest_params)*]}
    };
}
