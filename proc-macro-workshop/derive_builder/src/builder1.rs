use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;
use syn::DeriveInput;

/// 根据输入的派生信息生成一个构建器结构体的定义
///
/// 此函数接收一个DeriveInput类型的input，其中包含了派生结构体的信息
/// 它基于这些信息生成一个对应构建器结构体的定义，并返回生成的TokenStream2
/// 主要用于在宏中构建派生结构体的构建器
///
/// # Parameters
/// - `input`: DeriveInput类型，包含了结构体的可见性、标识符和数据结构等信息。
///
/// # Returns
/// - `syn::Result<TokenStream2>`: 返回一个结果，包含生成的构建器模式代码流或错误信息。
pub fn expand(input: DeriveInput) -> syn::Result<TokenStream2> {
  // 获取结构体的可见性和标识符
  let vis = &input.vis;
  let input_ident = &input.ident;
  // 生成构建器结构体的标识符
  let builder_ident = Ident::new(&format!("{}Builder", input_ident), Span::call_site());

  // 使用quote宏生成构建器结构体的定义，并将其包装在Ok中返回
  Ok(quote! {
    #vis struct #builder_ident {
    }
  })
}

