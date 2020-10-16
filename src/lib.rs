//! # derive `Display` for simple enums
//!
//! You can derive the `Display` trait for simple enums.
//!
//! Actually, the most complex enum definition that this crate supports is like this one:
//!
//! ```rust,ignore
//! #[derive(Display)]
//! pub enum FooBar {
//!     Foo,
//!     Bar(),
//!     FooBar(i32), // some wrapped type which implements Display
//! }
//! ```
//!
//! The above code will be expanded (roughly, without the enum definition) into this code:
//!
//! ```rust,ignore
//! impl Display for FooBar {
//!     fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
//!         match *self {
//!             FooBar::Foo => f.write_str("Foo"),
//!             FooBar::Bar => f.write_str("Bar()"),
//!             FooBar::FooBar(ref inner) => ::std::fmt::Display::fmt(inner, f),
//!         }
//!     }
//! }
//! ```
//!
//! ## Examples
//!
//! ```rust,ignore
//! #[macro_use]
//! extern crate enum_display_derive;
//!
//! use std::fmt::Display;
//!
//! #[derive(Display)]
//! enum FizzBuzz {
//!    Fizz,
//!    Buzz,
//!    FizzBuzz,
//!    Number(u64),
//! }
//!
//! fn fb(i: u64) {
//!    match (i % 3, i % 5) {
//!        (0, 0) => FizzBuzz::FizzBuzz,
//!        (0, _) => FizzBuzz::Fizz,
//!        (_, 0) => FizzBuzz::Buzz,
//!        (_, _) => FizzBuzz::Number(i),
//!    }
//! }
//!
//! fn main() {
//!    for i in 0..100 {
//!        println!("{}", fb(i));
//!    }
//! }
//! ```
//!

extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

#[proc_macro_derive(Display, attributes(display))]
#[doc(hidden)]
pub fn display(input: TokenStream) -> TokenStream {
    // Parse the string representation
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    match ast.data {
        syn::Data::Enum(ref enum_data) => {
            let name = &ast.ident;
            impl_display(name, enum_data).into()
        }
        _ => panic!("#[derive(Display)] works only on enums"),
    }
}

fn impl_display(name: &syn::Ident, data: &syn::DataEnum) -> proc_macro2::TokenStream {
    let variants = data.variants.iter()
        .map(|variant| impl_display_for_variant(name, variant));

    quote! {
        impl Display for #name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
                match *self {
                    #(#variants)*
                }
            }
        }
    }
}

fn impl_display_for_variant(name: &syn::Ident, variant: &syn::Variant) -> proc_macro2::TokenStream {
    let id = &variant.ident;

    let variant_str = match variant
        .attrs
        .iter()
        .find(|a| a.path.get_ident().map_or(false, |i| i == "display"))
    {
        Some(display_attr) => {
            let lit: proc_macro2::Literal = display_attr.parse_args().unwrap();
            lit.to_string().trim_matches('"').to_string()
        }
        None => id.to_string(),
    };

    match variant.fields {
        syn::Fields::Unit => {
            quote! {
                #name::#id => {
                    f.write_str(#variant_str)
                }
            }
        }
        syn::Fields::Unnamed(ref fields) => {
            match fields.unnamed.len() {
                0 => {
                    quote! {
                        #name::#id() => {
                            f.write_str(#variant_str)?;
                            f.write_str("()")
                        }
                    }
                }
                1 => {
                    quote! {
                        #name::#id(ref inner) => {
                            ::std::fmt::Display::fmt(inner, f)
                        }
                    }
                }
                _ => {
                    panic!("#[derive(Display)] does not support tuple variants with more than one \
                            fields")
                }
            }
        }
        _ => panic!("#[derive(Display)] works only with unit and tuple variants"),
    }
}
