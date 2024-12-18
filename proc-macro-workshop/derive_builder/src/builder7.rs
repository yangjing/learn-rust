use crate::helper::{extract_inner_ty, extract_struct_fields};
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{DeriveInput, Error, Field, LitStr, Meta, Type, Visibility};

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
  let build_fn = make_build_fn(vis, &input_ident, &builder_fields);

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
      #build_fn
    }
  })
}

pub enum FieldType {
  Plain(Type),
  Optional(Type),
  Repeated(Ident, Type),
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

  /// 从`Field`类型中尝试创建一个`BuilderField`。
  ///
  /// 此函数主要用于解析字段上的属性，以确定如何在构建器模式中处理该字段。
  /// 它会检查字段是否标记有特定的`builder`属性，并根据该属性的参数决定字段的类型。
  ///
  /// # Parameters
  /// - `field`: 对结构体字段的引用，用于提取属性和类型信息。
  ///
  /// # Returns
  /// - `syn::Result<Self>`: 返回一个构建器字段的实例，如果解析过程中遇到错误，则返回错误。
  fn try_from(field: &Field) -> syn::Result<Self> {
    // 初始化`each`标识符为None，用于存储重复字段的标识符。
    let mut each = None::<Ident>;

    // 遍历字段的所有属性，寻找名为`builder`的属性。
    for attr in field.attrs.iter() {
      // 如果属性不是`builder`，则跳过。
      if !attr.path().is_ident("builder") {
        continue;
      }

      // 定义期望的属性格式。
      let expected = r#"expected `builder(each = "...")`"#;

      // 解析属性的元数据，确保它是一个列表类型。
      let meta = match &attr.meta {
        Meta::List(meta) => meta,
        meta => return Err(Error::new_spanned(meta, expected)),
      };

      // 解析嵌套的元数据，寻找`each`参数。
      meta.parse_nested_meta(|nested| {
        // 如果找到`each`参数，则解析其值并更新`each`变量。
        if nested.path.is_ident("each") {
          let lit: LitStr = nested.value()?.parse()?;
          each = Some(lit.parse()?);
          Ok(())
        } else {
          // 如果遇到未知参数，则返回错误。
          Err(Error::new_spanned(meta, expected))
        }
      })?; // 这里的 ? 处理 `parse_nested_meta` 调用后的错误并提前返回
    }

    // 克隆字段的标识符，unwrap是因为字段在结构体中总是有标识符的。
    let ident = field.ident.clone().unwrap();

    // 如果`each`有值，说明字段是重复类型。
    if let Some(each) = each {
      return Ok(BuilderField::new(ident, FieldType::Repeated(each, field.ty.clone())));
    }

    // 如果字段类型是`Option`，则提取内部类型并标记字段为可选类型。
    if let Type::Path(ty) = &field.ty {
      if let Some(segment) = ty.path.segments.last() {
        if segment.ident == "Option" {
          let inner_ty = extract_inner_ty(&segment.arguments)?;
          return Ok(BuilderField::new(ident, FieldType::Optional(inner_ty.clone())));
        }
      }
    }

    // 如果字段不是重复类型也不是可选类型，则标记为普通类型。
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
        FieldType::Repeated(_, ty) => quote!(#ty),
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
      let init = match &field.ty {
        FieldType::Plain(_) | FieldType::Optional(_) => quote!(Option::None),
        FieldType::Repeated(_, ty) => quote!(<#ty>::new()),
      };
      quote! {
        #ident: #init,
      }
    })
    .collect()
}

fn make_setters(vis: &Visibility, fields: &[BuilderField]) -> TokenStream2 {
  fields
    .iter()
    .map(|field| {
      let ident = &field.ident;
      let plain_store = quote!(self.#ident = Option::Some(#ident));
      let inner = |ty| quote!(<#ty as std::iter::IntoIterator>::Item);
      let (fn_name, arg_name, arg_ty, store) = match &field.ty {
        FieldType::Plain(ty) => (ident, ident, quote!(#ty), plain_store),
        FieldType::Optional(ty) => (ident, ident, quote!(#ty), plain_store),
        FieldType::Repeated(each, ty) => (each, each, inner(ty), quote!(self.#ident.push(#each)))
      };
      quote! {
        #vis fn #fn_name(&mut self, #arg_name: #arg_ty) -> &mut Self {
          #store;
          self
        }
      }
    })
    .collect()
}

// 根据给定的可见性、标识符和构建器字段生成构建函数的Token流
fn make_build_fn(vis: &Visibility, input_ident: &Ident, fields: &[BuilderField]) -> TokenStream2 {
  // 迭代字段，为每个非可选字段生成检查逻辑
  let required_field_checks = fields.iter().filter_map(|field| {
    // 获取字段标识符
    let ident = &field.ident;
    // 准备错误消息
    let error = format!("value is not set: {}", ident);
    // 根据字段类型决定是否生成检查逻辑
    match &field.ty {
      FieldType::Plain(_) => Some(quote! {
        let #ident = self.#ident.take().ok_or_else(|| {
          Box::<dyn core::error::Error>::from(#error)
        })?;
      }),
      FieldType::Optional(_) | FieldType::Repeated(..) => None,
    }
  });

  // 迭代字段，生成字段赋值逻辑
  let field_assignment = fields.iter().map(|field| {
    // 获取字段标识符
    let ident = &field.ident;
    // 根据字段类型生成不同的赋值表达式
    let expr = match &field.ty {
      FieldType::Plain(_) => quote!(#ident),
      FieldType::Optional(_) => quote!(self.#ident.take()),
      FieldType::Repeated(_, ty) => quote!(core::mem::replace(&mut self.#ident, <#ty>::new()))
    };
    // 生成字段赋值逻辑
    quote! {
      #ident: #expr,
    }
  });

  // 生成并返回构建函数的完整Token流
  quote! {
    #vis fn build(&mut self) -> Result<#input_ident, Box<dyn core::error::Error>> {
      #(#required_field_checks)*

      Ok(#input_ident {
        #(#field_assignment)*
      })
    }
  }
}
