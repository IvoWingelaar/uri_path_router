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
///     "foo" => {
///         "a" => FooA,
///         "b" => FooB,
///     },
///     "bar" => {
///         "a" => BarA,
///         "b" => BarB {
///            "x" => X,
///            "y" => Y,
///         },
///     }
/// }
///
/// # fn main() {
/// // Inside a function you can match routes using:
/// assert_eq!(route("/foo/a"), Some(Route::FooA));
/// assert_eq!(route("/foo/a/b"), None);
/// assert_eq!(route("/bar/b"), Some(Route::BarB));
/// assert_eq!(route("/bar/b/"), None);
/// assert_eq!(route("/bar/b/x"), Some(Route::X));
/// assert_eq!(route("/bar/b/y"), Some(Route::Y));
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
