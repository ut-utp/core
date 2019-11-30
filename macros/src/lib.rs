//! Helper proc-macros.
//!
//! TODO!

// TODO: forbid
#![warn(
    bad_style,
    const_err,
    dead_code,
    improper_ctypes,
    legacy_directory_ownership,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    plugin_as_library,
    private_in_public,
    safe_extern_statics,
    unconditional_recursion,
    unused,
    unused_allocation,
    unused_lifetimes,
    unused_comparisons,
    unused_parens,
    while_true
)]
// TODO: deny
#![warn(
    missing_debug_implementations,
    intra_doc_link_resolution_failure,
    missing_docs,
    unsafe_code,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results,
    rust_2018_idioms
)]
#![doc(test(attr(deny(rust_2018_idioms, warnings))))]
#![doc(html_logo_url = "")] // TODO!

extern crate proc_macro;
use proc_macro2::Span;
use quote::quote_spanned;

/// Report an error with the given `span` and message.
pub(crate) fn spanned_err(span: Span, msg: impl Into<String>) -> proc_macro::TokenStream {
    let msg = msg.into();
    quote_spanned!(span.into() => {
        compile_error!(#msg);
    })
    .into()
}

// pub mod lifetime_to_identifier;

// pub use lifetime_to_identifier::lifetime_to_ident;

use proc_macro::TokenTree;
use quote::quote;
use syn::{parse::Parse, parse::ParseStream, parse::Result as ParseResult, Ident};
use syn::{parse_macro_input, Lifetime};

/// Takes a lifetime and turns it into an identifier.
#[proc_macro]
pub fn lifetime_to_ident(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(tokens as Lifetime);

    proc_macro::TokenStream::from(quote!(#input.ident))
}

struct LifetimeVarDecl {
    ident: Ident,
    rest: proc_macro2::TokenTree,
}

impl Parse for LifetimeVarDecl {
    fn parse(input: ParseStream<'_>) -> ParseResult<Self> {
        let lifetime = input.parse::<Lifetime>()?;

        let tree = input.cursor().token_tree().ok_or_else(|| {
            syn::Error::new(Span::call_site(), "Expected a lifetime and a token tree.")
        })?;

        Ok(LifetimeVarDecl {
            ident: lifetime.ident,
            rest: tree.0,
        })
    }
}

#[proc_macro]
pub fn create_label(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(tokens as LifetimeVarDecl);

    let ident = input.ident;
    let tree = input.rest;

    proc_macro::TokenStream::from(quote!(let #ident: Addr = #tree;))
}
