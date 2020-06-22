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
///     }
/// }
///
/// # fn main() {
/// // Inside a function you can match routes using:
/// assert_eq!(route("/foo"), None);
/// assert_eq!(route("/foo/a"), Some(Route::FooA));
/// assert_eq!(route("/foo/a/b"), None);
/// assert_eq!(route("/foo/b"), Some(Route::FooB));
/// assert_eq!(route("/bar"), None);
/// assert_eq!(route("/bar/a"), Some(Route::BarA));
/// assert_eq!(route("/bar/whatever"), Some(Route::Bar { x: "whatever" }));
/// assert_eq!(route("/bar/whatever/"), None);
/// assert_eq!(route("/bar/baz/x"), Some(Route::X { x: "baz" }));
/// assert_eq!(route("/bar/baz/y"), Some(Route::Y { x: "baz" }));
/// assert_eq!(route("/baz/a/b"), Some(Route::Baz));
/// assert_eq!(route("/wrong/b"), None);
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
