use itertools::intersperse;
use proc_macro::TokenStream as RawTokenStream;
use proc_macro2::{Punct, Span, TokenStream, TokenTree};
use quote::{__private::ext::RepToTokensExt, quote};
use syn::{
    parse_macro_input,
    spanned::Spanned,
    token::{Enum, Union},
    Attribute, Data, DataEnum, DataUnion, DeriveInput, Field, Ident, Lit, LitStr,
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
        std::iter::repeat(LitStr::new("{}={}", Span::call_site())).take(num_fields),
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
        let path = &attr.path.segments;
        let attr_ident = &path[0].ident;

        match attr_ident.to_string().as_str() {
            "skip" => {
                if !attr.tokens.is_empty() {
                    return Err(syn::Error::new(
                        attr.tokens.span(),
                        "skip does not expect any arguments",
                    ));
                }
                attr_info.skip = true;
                return Ok(attr_info);
            }
            "result" => attr_info.rename = Some(parse_result_attr(attr_ident, attr)?),
            _ => return Err(syn::Error::new(attr_ident.span(), "unimplemented")),
        }
    }

    Ok(attr_info)
}

fn parse_result_attr(attr_ident: &Ident, attr: &Attribute) -> Result<LitStr, syn::Error> {
    if attr.tokens.is_empty() {
        return Err(syn::Error::new(
            attr_ident.span(),
            "expected: `result(name = \"...\")`",
        ));
    }

    // Ensure we have some content to begin with
    let Some(TokenTree::Group(group)) = attr.tokens.clone().into_iter().next() else {
        return Err(syn::Error::new(
            attr.tokens.span(),
            "expected: `result(name = \"...\")`",
        ));
    };

    // Ensure the content is delimited by parentheses
    match group.delimiter() {
        proc_macro2::Delimiter::Parenthesis => {}
        _ => {
            return Err(syn::Error::new(
                group.delim_span().span(),
                "expected: `result(name = \"...\")`",
            ));
        }
    }

    // Extract content from group
    let mut content = match group.stream().next() {
        Some(cnt) => cnt,
        None => {
            return Err(syn::Error::new(
                group.span(),
                "expected: `result(name = \"...\")`",
            ));
        }
    }
    .clone()
    .into_iter();

    // "name" token
    match content.next() {
        Some(ident @ TokenTree::Ident(..)) if ident.to_string() == "name" => {}
        Some(token) => {
            return Err(syn::Error::new(token.span(), "expected \"name\""));
        }
        None => {
            return Err(syn::Error::new(
                group.span(),
                "expected: `result(name = \"...\")`",
            ));
        }
    }

    // The '='
    match content.next() {
        Some(TokenTree::Punct(p @ Punct { .. })) if p.as_char() == '=' => {}
        Some(token) => {
            return Err(syn::Error::new(token.span(), "expected \'=\'"));
        }
        None => {
            return Err(syn::Error::new(
                group.span(),
                "expected: `result(name = \"...\")`",
            ));
        }
    }

    // The rename string
    let rename_str = match content.next() {
        Some(TokenTree::Literal(lit)) => {
            let span = lit.span();
            match Lit::new(lit) {
                Lit::Str(str_lit) => str_lit,
                // If this is anything but a string literal, it's an error
                _ => {
                    return Err(syn::Error::new(span, "expected string literal identifier"));
                }
            }
        }
        Some(token) => {
            return Err(syn::Error::new(token.span(), "expected a string literal"));
        }
        None => {
            return Err(syn::Error::new(
                group.span(),
                "expected: `result(name = \"...\")`",
            ));
        }
    };

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
