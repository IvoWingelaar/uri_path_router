//! A procedural macro crate for creating fast URI path routing functions.
use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

mod matcher;

/// Create a routing function
///
/// ```
/// # use uri_path_router::route;
/// // Define a routing function with the following match-like syntax:
/// route! {
///     Route,
///     "foo" => {
///         "a" => FooA,
///         "b" => FooB,
///     },
///     "bar" => {
///         "a" => BarA,
///         x => Bar(x) {
///            "x" => X(x),
///            "y" => Y(x),
///         },
///     },
///     "baz" / "a" =>  {
///         "b" => Baz,
///     },
///     "long" / x / y / "z" => Long(x, y)
/// }
///
/// # fn main() {
/// // Inside a function you can match routes using:
/// assert_eq!(Route::try_from("/foo"), Err(()));
/// assert_eq!(Route::try_from("/foo/a"), Ok(Route::FooA));
/// assert_eq!(Route::try_from("/foo/a/b"), Err(()));
/// assert_eq!(Route::try_from("/foo/b"), Ok(Route::FooB));
/// assert_eq!(Route::try_from("/bar"), Err(()));
/// assert_eq!(Route::try_from("/bar/a"), Ok(Route::BarA));
/// assert_eq!(Route::try_from("/bar/whatever"), Ok(Route::Bar { x: "whatever" }));
/// assert_eq!(Route::try_from("/bar/whatever/"), Err(()));
/// assert_eq!(Route::try_from("/bar/baz/x"), Ok(Route::X { x: "baz" }));
/// assert_eq!(Route::try_from("/bar/baz/y"), Ok(Route::Y { x: "baz" }));
/// assert_eq!(Route::try_from("/baz/a/b"), Ok(Route::Baz));
/// assert_eq!(Route::try_from("/long/a/b/z"), Ok(Route::Long { x: "a", y: "b" }));
/// assert_eq!(Route::try_from("/wrong/b"), Err(()));
/// # }
///
/// ```
#[proc_macro]
pub fn route(input: TokenStream) -> TokenStream {
    let matcher = parse_macro_input!(input as matcher::Matcher);

    let tokens = quote! {
        #matcher
    };

    TokenStream::from(tokens)
}
