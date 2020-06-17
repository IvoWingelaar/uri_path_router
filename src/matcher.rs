use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream, Result};
use syn::Token;

mod rule;

use rule::Rule;

pub(crate) struct Matcher {
    rules: Vec<Box<Rule>>,
}

impl Parse for Matcher {
    fn parse(input: ParseStream) -> Result<Self> {
        let rules = input
            .parse_terminated::<_, Token![,]>(Rule::parse)?
            .into_iter()
            .map(Box::new)
            .collect();

        Ok(Matcher { rules })
    }
}

impl ToTokens for Matcher {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let mut names = Vec::new();

        for i in &self.rules {
            i.add_names(&mut names);
        }

        let rules = self.rules.iter().map(|x| {
            let a = &x.pattern;

            quote! {
                Some(#a) => #x
            }
        });

        let x = quote! {
            let r = match segments.next() {
                #(#rules),*,
                _ => return None,
            };

            Some(r)

        };

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

                #x
            }
        };

        expand.to_tokens(tokens);
    }
}
