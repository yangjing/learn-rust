use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod builder1;
mod builder2;
mod builder3;
mod builder4;
mod builder5;
mod builder6;
mod builder7;
mod builder9;
mod helper;

#[proc_macro_derive(Builder1)]
pub fn derive_builder1(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);
  let expanded = builder1::expand(input).unwrap_or_else(|err| err.to_compile_error());
  expanded.into()
}

#[proc_macro_derive(Builder2)]
pub fn derive_builder2(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);
  let expanded = builder2::expand(input).unwrap_or_else(|err| err.to_compile_error());
  expanded.into()
}

#[proc_macro_derive(Builder3)]
pub fn derive_builder3(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);
  let expanded = builder3::expand(input).unwrap_or_else(|err| err.to_compile_error());
  expanded.into()
}

#[proc_macro_derive(Builder4)]
pub fn derive_builder4(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);
  let expanded = builder4::expand(input).unwrap_or_else(|err| err.to_compile_error());
  expanded.into()
}

#[proc_macro_derive(Builder5)]
pub fn derive_builder5(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);
  let expanded = builder5::expand(input).unwrap_or_else(|err| err.to_compile_error());
  expanded.into()
}

#[proc_macro_derive(Builder6)]
pub fn derive_builder6(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);
  let expanded = builder6::expand(input).unwrap_or_else(|err| err.to_compile_error());
  expanded.into()
}

#[proc_macro_derive(Builder7, attributes(builder))]
pub fn derive_builder7(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);
  let expanded = builder7::expand(input).unwrap_or_else(|err| err.to_compile_error());
  expanded.into()
}

#[proc_macro_derive(Builder9, attributes(builder))]
pub fn derive_builder9(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);
  let expanded = builder9::expand(input).unwrap_or_else(|err| err.to_compile_error());
  expanded.into()
}
