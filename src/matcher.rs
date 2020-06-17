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
        let mut names = Vec::new();

        for i in self.rules.iter() {
            i.add_names(&mut names);
        }

        let rules = &self.rules;
        let rules = quote! { #rules };

        let expand = quote! {
            #[derive(Debug, PartialEq)]
            pub enum Route {
                #(#names),*
            }
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
