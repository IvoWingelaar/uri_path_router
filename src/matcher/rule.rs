use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream, Result};
use syn::{braced, token, Token};

#[derive(Debug)]
pub(crate) struct Rule {
    pub(crate) pattern: syn::LitStr,
    ty: Option<syn::Ident>,
    children: Vec<Box<Rule>>,
}

impl Rule {
    pub(crate) fn add_names(&self, names: &mut Vec<syn::Ident>) {
        if let Some(ty) = &self.ty {
            names.push(ty.clone());
        }

        for i in &self.children {
            i.add_names(names);
        }
    }
}

impl Parse for Rule {
    fn parse(input: ParseStream) -> Result<Self> {
        let pattern: syn::LitStr = input.parse()?;

        while input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            let _: syn::LitStr = input.parse()?;
        }

        input.parse::<Token![=>]>()?;

        let ty = input.parse::<syn::Ident>();

        let children = if input.peek(token::Brace) {
            let extra;
            braced!(extra in input);

            extra
                .parse_terminated::<_, Token![,]>(Rule::parse)?
                .into_iter()
                .map(Box::new)
                .collect()
        } else {
            Vec::new()
        };

        Ok(Rule {
            pattern,
            ty: ty.ok(),
            children,
        })
    }
}

impl ToTokens for Rule {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if self.children.len() > 0 {
            let rules = self.children.iter().map(|x| {
                let a = &x.pattern;
                let ty = &x.ty;

                // We are careful to avoid the tail of the path in a match
                // by forcing the next segment to be `None`.
                quote! {
                    Some(#a) => {
                        if segments.next().is_none() {
                            Route::#ty
                        } else {
                            return None
                        }
                    }
                }
            });

            let x = quote! {
                {
                    let r = match segments.next() {
                        #(#rules),*,
                        _ => return None,
                    };

                    r
                }
            };

            x.to_tokens(tokens);
        } else {
            let ty = &self.ty;
            let x = quote! { Route::#ty };

            x.to_tokens(tokens);
        }
    }
}
