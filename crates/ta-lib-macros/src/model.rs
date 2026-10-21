//! AST model and parser for `ta_indicators!` declarations.

use heck::{ToShoutySnakeCase, ToUpperCamelCase};
use syn::{
    Attribute, Ident, Result, Token, braced, bracketed,
    ext::IdentExt,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum OptKind {
    Int,
    Real,
    MaType,
}

#[derive(Clone)]
pub struct OptParam {
    pub name: Ident,
    pub kind: OptKind,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum OutKind {
    F64,
    I32,
}

#[derive(Clone)]
pub struct OutParam {
    pub name: Ident,
    pub kind: OutKind,
}

pub struct IndicatorSpec {
    pub attrs: Vec<Attribute>,
    pub fn_name: Ident,
    pub c_ident: Ident,
    pub stream_ident: Ident,
    pub inputs: Vec<Ident>,
    pub opts: Vec<OptParam>,
    pub outs: Vec<OutParam>,
}

pub struct IndicatorList {
    pub items: Vec<IndicatorSpec>,
}

impl Parse for IndicatorSpec {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let fn_name: Ident = input.parse()?;
        let (c_ident, stream_ident) = if input.peek(Token![=>]) {
            input.parse::<Token![=>]>()?;
            let c_ident: Ident = input.parse()?;
            input.parse::<Token![,]>()?;
            let stream_ident: Ident = input.parse()?;
            (c_ident, stream_ident)
        } else {
            let base = fn_name.unraw().to_string();
            let c_ident = Ident::new(&base.to_shouty_snake_case(), fn_name.span());
            let stream_ident = Ident::new(
                &format!("{}Stream", base.to_upper_camel_case()),
                fn_name.span(),
            );
            (c_ident, stream_ident)
        };

        let content;
        braced!(content in input);

        // inputs: [a, b, ...]
        let in_kw: Ident = content.parse()?;
        if in_kw != "inputs" {
            return Err(syn::Error::new(in_kw.span(), "expected `inputs`"));
        }
        content.parse::<Token![:]>()?;
        let in_brackets;
        bracketed!(in_brackets in content);
        let inputs_punct: Punctuated<Ident, Token![,]> =
            in_brackets.parse_terminated(Ident::parse, Token![,])?;
        let inputs: Vec<Ident> = inputs_punct.into_iter().collect();
        content.parse::<Token![,]>()?;

        // opts: [name: kind, ...]
        let opt_kw: Ident = content.parse()?;
        if opt_kw != "opts" {
            return Err(syn::Error::new(opt_kw.span(), "expected `opts`"));
        }
        content.parse::<Token![:]>()?;
        let opt_brackets;
        bracketed!(opt_brackets in content);
        let mut opts = Vec::new();
        while !opt_brackets.is_empty() {
            let name: Ident = opt_brackets.parse()?;
            opt_brackets.parse::<Token![:]>()?;
            let ty_ident: Ident = opt_brackets.parse()?;
            let kind = match ty_ident.to_string().as_str() {
                "i32" | "int" => OptKind::Int,
                "f64" | "real" => OptKind::Real,
                "MAType" => OptKind::MaType,
                _ => {
                    return Err(syn::Error::new(
                        ty_ident.span(),
                        "expected `i32`, `f64`, or `MAType`",
                    ));
                }
            };
            opts.push(OptParam { name, kind });
            if opt_brackets.peek(Token![,]) {
                opt_brackets.parse::<Token![,]>()?;
            }
        }
        content.parse::<Token![,]>()?;

        // outs: [name: kind, ...]
        let out_kw: Ident = content.parse()?;
        if out_kw != "outs" {
            return Err(syn::Error::new(out_kw.span(), "expected `outs`"));
        }
        content.parse::<Token![:]>()?;
        let out_brackets;
        bracketed!(out_brackets in content);
        let mut outs = Vec::new();
        while !out_brackets.is_empty() {
            let name: Ident = out_brackets.parse()?;
            out_brackets.parse::<Token![:]>()?;
            let ty_ident: Ident = out_brackets.parse()?;
            let kind = match ty_ident.to_string().as_str() {
                "f64" => OutKind::F64,
                "i32" => OutKind::I32,
                _ => {
                    return Err(syn::Error::new(ty_ident.span(), "expected `f64` or `i32`"));
                }
            };
            outs.push(OutParam { name, kind });
            if out_brackets.peek(Token![,]) {
                out_brackets.parse::<Token![,]>()?;
            }
        }
        if content.peek(Token![,]) {
            content.parse::<Token![,]>()?;
        }

        Ok(Self {
            attrs,
            fn_name,
            c_ident,
            stream_ident,
            inputs,
            opts,
            outs,
        })
    }
}

impl Parse for IndicatorList {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut items = Vec::new();
        while !input.is_empty() {
            items.push(input.parse()?);
            if input.peek(Token![;]) {
                input.parse::<Token![;]>()?;
            }
        }
        Ok(Self { items })
    }
}
