use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream, Result};
use syn::{braced, token, Token};
use syn::{punctuated::Punctuated, Ident};

#[derive(Debug)]
pub enum Capture {
    Lit(syn::LitStr),
    Var(syn::Ident),
}

impl ToTokens for Capture {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let x = match self {
            Capture::Lit(x) => {
                quote! { #x }
            }
            Capture::Var(x) => {
                quote! { #x }
            }
        };

        x.to_tokens(tokens);
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Variant {
    ident: Ident,
    fields: Option<Punctuated<Ident, Token![,]>>,
    add_types: bool,
}

impl ToTokens for Variant {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if let Some(fields) = &self.fields {
            let ident = &self.ident;
            let iter = fields.into_iter();

            let x = if self.add_types {
                quote! {
                    #ident { #( #iter: &'a str, )* }
                }
            } else {
                quote! {
                    #ident { #( #iter, )* }
                }
            };

            x.to_tokens(tokens);
        } else {
            let ident = &self.ident;

            let x = quote! {
                #ident
            };

            x.to_tokens(tokens);
        }
    }
}

#[derive(Debug)]
pub(crate) struct Captures(Vec<Capture>);

impl std::ops::Deref for Captures {
    type Target = Vec<Capture>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Parse for Captures {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut pattern = Vec::new();

        let capture = if input.peek(syn::LitStr) {
            Capture::Lit(input.parse()?)
        } else {
            Capture::Var(input.parse()?)
        };

        pattern.push(capture);

        while input.peek(Token![/]) {
            input.parse::<Token![/]>()?;

            let capture = if input.peek(syn::LitStr) {
                Capture::Lit(input.parse()?)
            } else {
                Capture::Var(input.parse()?)
            };

            pattern.push(capture);
        }

        Ok(Captures(pattern))
    }
}

#[derive(Debug)]
pub(crate) struct Rule {
    pub(crate) pattern: Captures,
    var: Option<Variant>,
    route_id: Option<Ident>,

    children: Rules,
}

impl Rule {
    pub(crate) fn add_variant(&self, variants: &mut Vec<Variant>) -> bool {
        let mut has_lifetime = false;
        if let Some(var) = &self.var {
            let mut v = var.clone();
            v.add_types = true;

            if let Some(fields) = &v.fields {
                if !fields.is_empty() {
                    has_lifetime = true;
                }
            }

            variants.push(v);
        }

        for i in self.children.iter() {
            if i.add_variant(variants) {
                has_lifetime = true;
            }
        }

        has_lifetime
    }

    fn set_route_id(&mut self, id: Ident) {
        self.route_id = Some(id.clone());

        self.children.set_route_id(id);
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

impl Rules {
    pub(crate) fn set_route_id(&mut self, id: Ident) {
        for i in &mut self.0 {
            i.set_route_id(id.clone());
        }
    }
}

impl Parse for Rule {
    fn parse(input: ParseStream) -> Result<Self> {
        let pattern = input.parse()?;

        input.parse::<Token![=>]>()?;

        let ty = input.parse::<syn::Ident>();

        let fields = if input.peek(token::Paren) {
            let fields;

            syn::parenthesized!(fields in input);

            Some(fields.parse_terminated(Ident::parse)?)
        } else {
            None
        };

        let var = if let Ok(ident) = ty {
            Some(Variant {
                ident,
                fields,
                add_types: false,
            })
        } else {
            None
        };

        let children = if input.peek(token::Brace) {
            let children;
            braced!(children in input);
            children.parse()?
        } else {
            Default::default()
        };

        Ok(Rule {
            pattern,
            var,
            route_id: None,
            children,
        })
    }
}

impl ToTokens for Rule {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ty = &self.var;
        let children = &self.children;

        let route_id = self.route_id.clone().unwrap();

        let x = if ty.is_some() {
            if children.len() == 0 {
                // We are careful to avoid the tail of the path in a match
                // by forcing the next segment to be `None`.
                quote! {
                    if segments.next().is_none() {
                        #route_id::#ty
                    } else {
                        return None
                    }
                }
            } else {
                quote! {
                    let next = segments.next();
                    if next.is_none() {
                        #route_id::#ty
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

        let mut prev = quote! {
            Some(#a) => {
                #x
            }
        };

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
