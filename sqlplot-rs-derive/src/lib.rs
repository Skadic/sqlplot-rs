use itertools::intersperse;
use proc_macro::TokenStream as RawTokenStream;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    parse_macro_input,
    spanned::Spanned,
    token::{Enum, Union},
    Attribute, Data, DataEnum, DataUnion, DeriveInput, Expr, ExprAssign, Field, Ident,
    Lit, LitStr, ExprLit,
};

#[proc_macro_derive(ResultLine, attributes(result, skip))]
pub fn derive(tokens: RawTokenStream) -> RawTokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);
    let data = match input.data {
        Data::Struct(d) => d,
        Data::Enum(DataEnum {
            enum_token: Enum { span },
            ..
        })
        | Data::Union(DataUnion {
            union_token: Union { span },
            ..
        }) => {
            return syn::Error::new(span, "deriving ResultLine is only possible on structs")
                .to_compile_error()
                .into();
        }
    };

    let struct_name = input.ident;
    let fields = match data.fields {
        syn::Fields::Named(named_fields) => named_fields,
        _ => {
            return syn::Error::new(
                data.fields.span(),
                "deriving ResultLine is only possible on structs with named fields",
            )
            .to_compile_error()
            .into();
        }
    };

    result_line_impl(struct_name, fields.named).into()
}

fn result_line_impl(struct_name: Ident, fields: impl IntoIterator<Item = Field>) -> TokenStream {
    let fields = fields.into_iter();

    // Map the fields to a pair of "stringified names" and the value, separated by a comma
    let mut fields = match fields.map(process_field).collect::<Result<Vec<_>, _>>() {
        Ok(v) => v,
        Err(e) => {
            return e.to_compile_error();
        }
    };
    fields.retain(|tk| !tk.is_empty());
    let num_fields = fields.len();

    let brackets = intersperse(
        std::iter::repeat(LitStr::new("{}={:?}", Span::call_site())).take(num_fields),
        LitStr::new(" ", Span::call_site()),
    );

    quote! {
        impl sqlplot_rs_core::ResultLine for #struct_name {
            fn to_result_line(&self) -> String {
                format!(concat!("RESULT ", #(#brackets),*), #(#fields),*)
            }
        }
    }
}

fn process_field(field: Field) -> Result<TokenStream, syn::Error> {
    let ident = field.ident.unwrap();

    let attrs = field.attrs;
    let attr_info = match parse_field_attr(&attrs) {
        Ok(info) => info,
        Err(comp_err) => return Err(comp_err),
    };

    Ok(if attr_info.skip {
        TokenStream::new()
    } else if let Some(rename) = attr_info.rename {
        quote! { #rename, self.#ident }
    } else {
        quote! { stringify!(#ident), self.#ident }
    })
}

#[derive(Default)]
struct FieldAttr {
    rename: Option<LitStr>,
    skip: bool,
}

fn parse_field_attr(attrs: &[Attribute]) -> Result<FieldAttr, syn::Error> {
    let mut attr_info = FieldAttr::default();
    if attrs.is_empty() {
        return Ok(attr_info);
    }

    // Handle all attributes
    for attr in attrs {
        let path = &attr.path().segments;
        let attr_ident = &path[0].ident;

        match attr_ident.to_string().as_str() {
            "skip" => {
                attr.meta.require_path_only()?;
                attr_info.skip = true;
                return Ok(attr_info);
            }
            "result" => attr_info.rename = Some(parse_result_attr(attr)?),
            _ => return Err(syn::Error::new(attr_ident.span(), "unimplemented")),
        }
    }

    Ok(attr_info)
}

fn parse_result_attr(attr: &Attribute) -> Result<LitStr, syn::Error> {
    attr.meta.require_list()?;
    let name_expr = attr.parse_args::<ExprAssign>()?;
    // The left side must simply be "name"
    match name_expr.left.as_ref() {
        Expr::Path(ident) => {
            let name_ident = Ident::new("name", ident.span());
            // This should only be the name identifier
            if !ident.path.is_ident(&name_ident) {
                return Err(syn::Error::new(ident.span(), "expected `name`"));
            }
        }
        // If there is some other expression, this is wrong
        _ => {
            return Err(syn::Error::new(
                name_expr.span(),
                "expected `name = \"...\"`",
            ))
        }
    }

    let rename_str = match name_expr.right.as_ref() {
        Expr::Lit(ExprLit { lit: Lit::Str(lit), ..}) => lit,
        ex => {
            return Err(syn::Error::new(
                ex.span(),
                "expected string literal",
            ))
        }
    }.clone();

    let rename_str_value = rename_str.value();
    let rename_str_span = rename_str.span();

    if rename_str_value.is_empty() {
        return Err(syn::Error::new(rename_str_span, "may not be empty"));
    }

    let first_char = rename_str_value.chars().next().unwrap();

    // rename must start with an alphabetic character or underscore
    if first_char != '_' && !first_char.is_ascii_alphabetic() {
        return Err(syn::Error::new(
            rename_str_span,
            "must start with an alphabetic character or underscore",
        ));
    }

    // rename may not contain whitespace
    if rename_str_value.chars().any(char::is_whitespace) {
        return Err(syn::Error::new(
            rename_str_span,
            "may not contain whitespace",
        ));
    }

    // rename must be alphanumeric or underscore
    if rename_str_value
        .chars()
        .any(|c| !c.is_ascii_alphanumeric() && c != '_')
    {
        return Err(syn::Error::new(
            rename_str_span,
            "may only contain ascii alphanumeric characters and underscores",
        ));
    }

    Ok(rename_str)
}
