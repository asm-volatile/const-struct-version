#![doc = include_str!("../README.md")]
#![deny(clippy::all, clippy::pedantic)]
#![allow(clippy::uninlined_format_args, clippy::needless_doctest_main)]

use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Attribute, Data, DeriveInput, Fields, GenericParam, parse_macro_input, spanned::Spanned,
    token::Brace,
};

/// Derive macro implementation for the `StructVersion` trait
#[proc_macro_derive(StructVersion)]
pub fn derive_struct_version(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = &input.ident;

    // Process item-level attributes and generate hash update code based on type
    let (item_attrs, item_code) = match &input.data {
        // Handle struct types
        Data::Struct(data) => {
            let fields = match &data.fields {
                Fields::Named(fields) => Some(&fields.named),
                Fields::Unnamed(unnamed) => Some(&unnamed.unnamed),
                Fields::Unit => None,
            };

            // Generate code for struct fields
            match fields {
                Some(fields) => {
                    let field_code = generate_struct_fields_code(&syn::FieldsNamed {
                        named: fields.clone(),
                        brace_token: Brace::default(),
                    });
                    (process_attrs(&input.attrs), field_code)
                }
                None => (process_attrs(&input.attrs), proc_macro2::TokenStream::new()),
            }
        }
        // Handle enum types
        Data::Enum(data_enum) => (
            process_attrs(&input.attrs),
            generate_enum_variants_code(&data_enum.variants),
        ),
        // Handle union types
        Data::Union(_) => {
            unimplemented!("Unions are not supported - you must implement StructVersion manually.")
        }
    };

    // Add StructVersion bounds to all generic type parameters
    let mut generics = input.generics.clone();
    for param in &mut generics.params {
        if let GenericParam::Type(type_param) = param {
            type_param.bounds.push(syn::parse_quote!(StructVersion));
        }
    }
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Generate final implementation code
    let version_impl = quote! {
        #[doc(hidden)]
        const _: () = {
            extern crate const_struct_version as _const_struct_version;
            use _const_struct_version::__private::sha1::Digest as _;

            /// Automatically derived implementation of StructVersion
            #[automatically_derived]
            impl #impl_generics _const_struct_version::StructVersion for #ident #ty_generics #where_clause {
                fn version() -> String {
                    let mut hasher = _const_struct_version::__private::sha1::Sha1::new();
                    #( _const_struct_version::__private::execute_if_serde_enabled(&mut hasher, |hasher| hasher.update(#item_attrs)); )*
                    #item_code
                    format!("{:x}", hasher.finalize())
                }
            }

            fn version_cached() -> &'static str {
                extern crate const_struct_version as _const_struct_version;
                static VERSION: ::std::sync::OnceLock<String> = ::std::sync::OnceLock::new();
                VERSION.get_or_init(|| version())
            }
        };
    };

    version_impl.into()
}

/// Generate hash update code for struct fields
fn generate_struct_fields_code(fields: &syn::FieldsNamed) -> proc_macro2::TokenStream {
    let field_code = fields.named.iter().enumerate().map(|(index, field)| {
        let field_name_str = field.ident.as_ref().map(std::string::ToString::to_string).unwrap_or(index.to_string());
        let field_attrs = process_attrs(&field.attrs);
        let field_ty = &field.ty;

        quote! {
            hasher.update(#field_name_str);
            #( _const_struct_version::__private::execute_if_serde_enabled(&mut hasher, |hasher| hasher.update(#field_attrs)); )*
            hasher.update(<#field_ty as _const_struct_version::StructVersion>::version().as_bytes());
        }
    });

    quote! { #( #field_code )* }
}

/// Generate hash update code for enum variants (handles both named and unnamed fields)
fn generate_enum_variants_code(
    variants: &syn::punctuated::Punctuated<syn::Variant, syn::Token![,]>,
) -> proc_macro2::TokenStream {
    let variant_code = variants.iter().map(|variant| {
        let variant_name = variant.ident.to_string();
        let variant_attrs = process_attrs(&variant.attrs);

        // Handle discriminant (e.g., Variant = 1)
        let disc_code = if let Some((eq_token, expr)) = &variant.discriminant {
            let eq_ts = eq_token.to_token_stream();
            let expr_ts = expr.to_token_stream();
            let disc_str = format!("{}{}", eq_ts, expr_ts);
            quote! { hasher.update(#disc_str); }
        } else {
            proc_macro2::TokenStream::new()
        };

        // Generate code for variant fields (both named and unnamed)
        let fields_code = variant.fields.iter().enumerate().map(|(idx, field)| {
            let field_name = match &field.ident {
                Some(name) => name.to_string(),
                None => idx.to_string(),
            };
            let field_attrs = process_attrs(&field.attrs);
            let field_ty = &field.ty;

            quote! {
                hasher.update(#field_name);
                #( _const_struct_version::__private::execute_if_serde_enabled(&mut hasher, |hasher| hasher.update(#field_attrs)); )*
                hasher.update(<#field_ty as _const_struct_version::StructVersion>::version().as_bytes());
            }
        });

        quote! {
            hasher.update(#variant_name);
            #( _const_struct_version::__private::execute_if_serde_enabled(&mut hasher, |hasher| hasher.update(#variant_attrs)); )*
            #disc_code
            #( #fields_code )*
        }
    });
    quote! { #( #variant_code )* }
}

/// Convert serde attributes to string literals for hashing
fn process_attrs(attrs: &[Attribute]) -> Vec<syn::LitStr> {
    attrs
        .iter()
        .filter(|attr| attr.path().is_ident("serde"))
        .map(|attr| {
            let path = attr.path();
            let tokens = quote! { #attr };
            let attr_ts = quote! { #path #tokens };
            let attr_str = attr_ts.to_string();
            syn::LitStr::new(&attr_str, attr.span())
        })
        .collect()
}
