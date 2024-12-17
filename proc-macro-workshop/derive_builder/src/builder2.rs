use crate::helper::extract_struct_fields;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{DeriveInput, Field, Type};

/// 根据输入的派生结构体生成一个构建器模式的实现。
///
/// 此函数接受一个DeriveInput类型的参数，其中包含了派生结构体的信息，
/// 包括可见性、标识符、数据结构等。函数会基于这些信息生成一个构建器结构体，
/// 并实现构建方法。
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

  // 解析结构体中的字段，确保其为命名字段
  let fields = extract_struct_fields(input.data)?;

  // 将字段信息转换为构建器字段信息
  let builder_fields: Vec<BuilderField> =
    fields.named.iter().map(BuilderField::try_from).collect::<syn::Result<_>>()?;

  // 生成构建器结构体的字段存储代码
  let storage = make_storage(&builder_fields);
  // 生成构建器结构体的初始化代码
  let initializer = make_initializer(&builder_fields);

  // 组合生成的代码片段，返回构建器模式的实现
  Ok(quote! {
    #vis struct #builder_ident {
      #storage
    }

    impl #input_ident {
      #vis fn builder() -> #builder_ident {
        #builder_ident {
          #initializer
        }
      }
    }
  })
}


/// 定义字段类型枚举，用于描述字段是否可选或必填
enum FieldType {
  /// 普通字段类型，直接包含字段的类型信息
  Plain(Type),
  /// 可选字段类型，表示字段可以不存在
  Optional(Type),
}

/// 构建器字段结构体，用于描述构建器中的字段信息
struct BuilderField {
  /// 字段标识符，用于在代码中引用该字段
  ident: Ident,
  /// 字段类型
  ty: FieldType,
}


impl BuilderField {
  fn new(ident: Ident, ty: FieldType) -> Self {
    BuilderField { ident, ty }
  }

  fn try_from(field: &Field) -> syn::Result<Self> {
    let ident = field.ident.clone().unwrap();

    if let Type::Path(ty) = &field.ty {
      if ty.path.segments.last().unwrap().ident == "Option" {
        return Ok(BuilderField::new(ident, FieldType::Optional(field.ty.clone())));
      }
    }

    Ok(BuilderField::new(ident, FieldType::Plain(field.ty.clone())))
  }
}

/// 根据提供的字段信息生成存储结构
///
/// 该函数接受一个BuilderField类型的切片作为输入，遍历每个字段，
/// 并根据字段的类型生成相应的存储结构。对于普通类型字段，生成一个该类型的Option；
/// 对于Optional类型字段，直接使用其内部类型。
///
/// # 参数
/// - `fields`: &[BuilderField] - 一组字段信息，包含字段的标识符和类型
///
/// # 返回值
/// - `TokenStream2` - 生成的存储结构，以TokenStream的形式返回
fn make_storage(fields: &[BuilderField]) -> TokenStream2 {
  fields
    .iter()
    .map(|field| {
      // 获取字段的标识符
      let ident = &field.ident;
      // 根据字段类型生成存储结构
      let storage = match &field.ty {
        FieldType::Plain(ty) => quote!(Option<#ty>),
        FieldType::Optional(ty) => quote!(#ty),
      };
      // 生成字段的定义代码
      quote! {
        #ident: #storage,
      }
    })
    // 将所有字段的定义代码合并成一个TokenStream
    .collect()
}


fn make_initializer(fields: &[BuilderField]) -> TokenStream2 {
  fields
    .iter()
    .map(|field| {
      let ident = &field.ident;
      quote! {
        #ident: None,
      }
    })
    .collect()
}
