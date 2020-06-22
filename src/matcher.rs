use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream, Result};

mod rule;

use rule::Rules;

pub(crate) struct Matcher {
    rules: Rules,
}

impl Parse for Matcher {
    fn parse(input: ParseStream) -> Result<Self> {
        let rules = input.parse()?;

        Ok(Matcher { rules })
    }
}

impl ToTokens for Matcher {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let mut variants = Vec::new();

        let mut has_lifetime = false;

        for i in self.rules.iter() {
            if i.add_variant(&mut variants) {
                has_lifetime = true;
            }
        }

        let enum_definition = if has_lifetime {
            quote! {
                #[derive(Debug, PartialEq)]
                pub enum Route<'a> {
                    #(#variants),*
                }
            }
        } else {
            quote! {
                #[derive(Debug, PartialEq)]
                pub enum Route {
                    #(#variants),*
                }
            }
        };

        let rules = &self.rules;
        let rules = quote! { #rules };

        let expand = quote! {
            #enum_definition
            pub fn route(path: &str) -> Option<Route> {
                let mut segments = path.split("/");

                if segments.next().is_none() {
                    return None;
                }

                let next = segments.next();

                let r = #rules;

                Some(r)
            }
        };

        expand.to_tokens(tokens);
    }
}
