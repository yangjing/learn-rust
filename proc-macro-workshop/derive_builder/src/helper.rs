use proc_macro2::Span;
use syn::{Data, Error, Fields, FieldsNamed, GenericArgument, PathArguments, Type};

/// 从路径参数中提取最内层的类型
///
/// 此函数旨在处理泛型参数，并从中提取出最内层的类型信息
/// 它主要用于解析和处理泛型相关的语法结构
///
/// # Parameters
/// - `arguments`: 一个借用的路径参数切片，用于从中提取类型信息
///
/// # Returns
/// - `syn::Result<&Type>`: 返回一个结果类型，包含对提取出的最内层类型的引用。如果提取失败，将返回一个描述错误原因的结果
pub fn extract_inner_ty(arguments: &PathArguments) -> syn::Result<&Type> {
  // 匹配路径参数的类型
  match &arguments {
    // 当路径参数是尖括号包围的泛型参数时
    PathArguments::AngleBracketed(generic_arg) => {
      // 进一步匹配泛型参数中的最后一个参数
      match generic_arg.args.last().unwrap() {
        // 如果最后一个参数是类型参数，则返回该类型
        GenericArgument::Type(inner_ty) => Ok(inner_ty),
        // 如果最后一个参数不是类型参数，则返回错误
        arg => return Err(Error::new_spanned(arg, "unexpected generic argument")),
      }
    }
    // 当路径参数不是尖括号包围的泛型参数时，返回错误
    path => return Err(Error::new_spanned(path, "unexpected path arguments")),
  }
}

/// 从给定的数据中提取结构体的命名字段
///
/// 此函数旨在处理特定的AST（抽象语法树）节点，具体来说是处理结构体定义。
/// 它尝试从提供的数据中提取出结构体的命名字段。如果数据不满足预期格式，函数将返回一个错误
///
///
/// # Parameters
/// - `data`: 一个结构体数据，期望是一个包含命名字段的结构体定义
///
/// # Returns
/// - `syn::Result<FieldsNamed>`: 如果成功提取到命名字段，则返回一个包含这些字段的`FieldsNamed`对象。否则，返回一个描述性错误
pub fn extract_struct_fields(data: Data) -> syn::Result<FieldsNamed> {
  // 根据数据类型进行匹配，我们只关心结构体类型的数据
  match data {
    // 当数据是结构体类型时，进一步检查其字段类型
    Data::Struct(s) => match s.fields {
      // 如果字段是命名字段，直接返回这些字段
      Fields::Named(fields) => Ok(fields),
      // 如果字段类型不是命名字段，返回错误，指明期望的是命名字段
      fields => return Err(Error::new_spanned(fields, "expected named fields")),
    },
    // 当数据类型不是结构体时，返回错误，指明期望的是结构体
    _ => return Err(Error::new(Span::call_site(), "expected struct")),
  }
}
