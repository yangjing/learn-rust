use crate::helper::{extract_inner_ty, extract_struct_fields};
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{DeriveInput, Field, Type, Visibility};

pub fn expand(input: DeriveInput) -> syn::Result<TokenStream2> {
  let vis = &input.vis;
  let input_ident = &input.ident;
  let builder_ident = Ident::new(&format!("{}Builder", input_ident), Span::call_site());

  let fields = extract_struct_fields(input.data)?;

  let builder_fields: Vec<BuilderField> =
    fields.named.iter().map(BuilderField::try_from).collect::<syn::Result<_>>()?;

  let storage = make_storage(&builder_fields);
  let initializer = make_initializer(&builder_fields);
  let setters = make_setters(vis, &builder_fields);

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

    impl #builder_ident {
      #setters
    }
  })
}

pub enum FieldType {
  Plain(Type),
  Optional(Type),
}

pub struct BuilderField {
  pub ident: Ident,
  pub ty: FieldType,
}

impl BuilderField {
  fn new(ident: Ident, ty: FieldType) -> Self {
    BuilderField { ident, ty }
  }
}

impl TryFrom<&Field> for BuilderField {
  type Error = syn::Error;

  /// 此函数主要用于解析给定的`Field`，并根据其类型创建一个`BuilderField`实例。
  /// 如果`Field`的类型是`Option<T>`，则会特别处理，以支持可选字段。
  ///
  /// # Parameters
  /// - `field`: &Field - 对一个字段的引用，包含字段的标识符和类型等信息。
  ///
  /// # Returns
  /// - `syn::Result<Self>` - 返回一个结果类型，如果成功，将包含创建的`BuilderField`实例。
  fn try_from(field: &Field) -> core::result::Result<Self, Self::Error> {
    // 克隆字段的标识符，用于后续创建`BuilderField`。
    let ident = field.ident.clone().unwrap();

    // 检查字段的类型是否为`Type::Path`，这是支持类型解析的前提。
    if let Type::Path(ty) = &field.ty {
      // 获取类型路径的最后一个段，这是实际类型信息所在的位置。
      if let Some(segment) = ty.path.segments.last() {
        // 检查是否为`Option`类型，以决定字段的处理方式。
        if segment.ident == "Option" {
          // 提取`Option`内部的类型，并创建一个`BuilderField`，将其类型标记为可选。
          let inner_ty = extract_inner_ty(&segment.arguments)?;
          return Ok(BuilderField::new(ident, FieldType::Optional(inner_ty.clone())));
        }
      }
    }

    // 如果字段类型不是`Option`，则直接创建一个`BuilderField`，将其类型标记为普通类型。
    Ok(BuilderField::new(ident, FieldType::Plain(field.ty.clone())))
  }
}

fn make_storage(fields: &[BuilderField]) -> TokenStream2 {
  fields
    .iter()
    .map(|field| {
      let ident = &field.ident;
      let storage = match &field.ty {
        FieldType::Plain(ty) | FieldType::Optional(ty) => quote!(Option<#ty>),
      };
      quote! {
        #ident: #storage,
      }
    })
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

/// 遍历每个字段信息，根据字段的类型生成对应的设置器方法。
/// 对于普通类型和可选类型字段，生成的方法会将字段设置为`Some(value)`，
/// 并返回`&mut Self`以支持链式调用
///
/// # Parameters
/// - `vis`: 指定设置器方法的可见性修饰符，如`pub`或默认可见性
/// - `fields`: 包含一系列字段信息的切片，用于生成设置器方法
///
/// # Returns
/// - 返回一个`TokenStream2`，包含所有生成的设置器方法的源码
///
fn make_setters(vis: &Visibility, fields: &[BuilderField]) -> TokenStream2 {
  fields
    .iter()
    .map(|field| {
      // 获取字段的标识符
      let ident = &field.ident;
      // 根据字段类型获取其内部类型，无论是普通类型还是可选类型
      let ident_type = match &field.ty {
        FieldType::Plain(ty) => ty,
        FieldType::Optional(ty) => ty,
      };
      // 生成并返回设置器方法的Token流
      quote! {
        #vis fn #ident(&mut self, #ident: #ident_type) -> &mut Self {
          self.#ident = Some(#ident);
          self
        }
      }
    })
    .collect()
}
