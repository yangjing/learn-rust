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

/// 这个宏通过解析输入的结构体定义，自动生成一个builder结构体以及相关的构建方法。
/// 使用 `attributes(builder)` 参数指定，允许使用builder属性来定制生成的行为。
///
/// # Parameters
/// - `input`: 类型为TokenStream的输入，代表了要应用Builder7特性的结构体的Rust源代码。
/// # Returns
/// - `TokenStream`: 生成的Rust代码，包含了builder模式的实现。这个代码流可以直接编译或进一步处理。
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
