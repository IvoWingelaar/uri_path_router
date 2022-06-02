use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream, Result};
use syn::{braced, token, Token};
use syn::{punctuated::Punctuated, Ident};

#[derive(Debug, Clone)]
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
    doc: String,
}

impl Variant {
    fn variant_tokens(&self, tokens: &mut proc_macro2::TokenStream, output_doc: bool) {
        let x = if let Some(fields) = &self.fields {
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

            x
        } else {
            let ident = &self.ident;

            let x = quote! {
                #ident
            };

            x
        };

        let x = if output_doc {
            let msg = &self.doc;
            quote! {
                #[doc = #msg]
                #x
            }
        } else {
            x
        };

        x.to_tokens(tokens);
    }
}

impl ToTokens for Variant {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.variant_tokens(tokens, true);
    }
}

// Skips output tokens for the doc code.
struct SkipVariant<'a>(&'a Variant);

impl<'a> ToTokens for SkipVariant<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.0.variant_tokens(tokens, false);
    }
}

#[derive(Debug, Clone)]
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
    root: Captures,

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

            let mut doc = String::new();

            for i in &self.root.0 {
                let string = match i {
                    Capture::Lit(lit) => lit.value(),
                    Capture::Var(var) => format!("{{{}}}", var.to_string()),
                };
                doc.push_str(&format!("/{}", string));
            }

            for i in &self.pattern.0 {
                let string = match i {
                    Capture::Lit(lit) => lit.value(),
                    Capture::Var(var) => format!("{{{}}}", var.to_string()),
                };

                doc.push_str(&format!("/{}", string));
            }

            v.doc = format!(
                "This variant will match a URI path bescribed by the following pattern: `{}`",
                doc
            );

            variants.push(v);
        }

        for i in self.children.iter() {
            if i.add_variant(variants) {
                has_lifetime = true;
            }
        }

        has_lifetime
    }

    fn walk_tree(&mut self, root: &[Capture]) {
        self.root = Captures(root.into());
        let mut root: Vec<_> = root.into();

        root.extend(self.pattern.0.iter().cloned());
        self.children.walk_tree(&root);
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

    /// Breadth-first traversal to update the pattern roots
    pub(crate) fn walk_tree(&mut self, root: &[Capture]) {
        for i in &mut self.0 {
            i.walk_tree(root);
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
                doc: "".to_string(),
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
            root: Captures(Vec::new()),
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
            // This skips the documentation production.
            let ty = ty.as_ref().unwrap();
            let ty = SkipVariant(&ty);

            if children.len() == 0 {
                // We are careful to avoid the tail of the path in a match
                // by forcing the next segment to be `None`.
                quote! {
                    if segments.next().is_none() {
                        #route_id::#ty
                    } else {
                        return Err(())
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

        for i in self.pattern.iter().take(len - 1).rev() {
            prev = quote! {
                Some(#i) => {
                    let next = segments.next();
                    match next {
                        #prev,
                        _ => return Err(()),
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
                    _ => return Err(()),
                }
            }
        };

        x.to_tokens(tokens);
    }
}
