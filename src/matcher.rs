use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream, Result};

mod rule;

use rule::Rules;

pub(crate) struct Matcher {
    route_identifier: syn::Ident,
    rules: Rules,
}

impl Parse for Matcher {
    fn parse(input: ParseStream) -> Result<Self> {
        let route_identifier: syn::Ident = input.parse()?;

        input.parse::<syn::token::Comma>()?;

        let mut rules: Rules = input.parse()?;

        rules.set_route_id(route_identifier.clone());

        Ok(Matcher {
            route_identifier,
            rules,
        })
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

        let route_id = &self.route_identifier;

        let enum_definition = if has_lifetime {
            quote! {
                #[derive(Debug, PartialEq)]
                pub enum #route_id<'a> {
                    #(#variants),*
                }
            }
        } else {
            quote! {
                #[derive(Debug, PartialEq)]
                pub enum #route_id {
                    #(#variants),*
                }
            }
        };

        let rules = &self.rules;
        let rules = quote! { #rules };

        let expand = quote! {
            #enum_definition
            pub fn route(path: &str) -> Option<#route_id> {
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
