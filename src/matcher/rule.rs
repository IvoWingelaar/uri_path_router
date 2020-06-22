use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream, Result};
use syn::{braced, token, Token};

#[derive(Debug)]
pub(crate) struct Rule {
    pub(crate) pattern: Vec<syn::LitStr>,
    ty: Option<syn::Ident>,
    children: Rules,
}

impl Rule {
    pub(crate) fn add_names(&self, names: &mut Vec<syn::Ident>) {
        if let Some(ty) = &self.ty {
            names.push(ty.clone());
        }

        for i in self.children.iter() {
            i.add_names(names);
        }
    }
}

#[derive(Debug)]
pub(crate) struct Rules(Vec<Box<Rule>>);

impl std::ops::Deref for Rules {
    type Target = Vec<Box<Rule>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for Rules {
    fn default() -> Self {
        Rules(Vec::new())
    }
}

impl Parse for Rules {
    fn parse(input: ParseStream) -> Result<Self> {
        let rules = input
            .parse_terminated::<_, Token![,]>(Rule::parse)?
            .into_iter()
            .map(Box::new)
            .collect();

        Ok(Rules(rules))
    }
}

impl Parse for Rule {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut pattern = Vec::new();

        pattern.push(input.parse()?);

        while input.peek(Token![/]) {
            input.parse::<Token![/]>()?;
            pattern.push(input.parse()?);
        }

        input.parse::<Token![=>]>()?;

        let ty = input.parse::<syn::Ident>();

        let children = if input.peek(token::Brace) {
            let children;
            braced!(children in input);
            children.parse()?
        } else {
            Default::default()
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
        let ty = &self.ty;
        let children = &self.children;

        let x = if ty.is_some() {
            if children.len() == 0 {
                // We are careful to avoid the tail of the path in a match
                // by forcing the next segment to be `None`.
                quote! {
                    if segments.next().is_none() {
                        Route::#ty
                    } else {
                        return None
                    }
                }
            } else {
                quote! {
                    let next = segments.next();
                    if next.is_none() {
                        Route::#ty
                    } else {
                        #children
                    }
                }
            }
        } else {
            quote! {
                let next = segments.next();
                #children
            }
        };

        let a = &self.pattern.last().unwrap();

        let x = quote! {
            Some(#a) => {
                #x
            }
        };

        let mut prev = x;

        let len = self.pattern.len();

        for i in self.pattern.iter().take(len - 1) {
            prev = quote! {
                Some(#i) => {
                    let next = segments.next();
                    match next {
                        #prev,
                        _ => return None,
                    }
                }
            };
        }

        prev.to_tokens(tokens);
    }
}

impl ToTokens for Rules {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let rules = self.0.iter().map(|x| {
            quote! { #x }
        });

        let x = quote! {
            {
                match next {
                    #(#rules),*,
                    _ => return None,
                }
            }
        };

        x.to_tokens(tokens);
    }
}
