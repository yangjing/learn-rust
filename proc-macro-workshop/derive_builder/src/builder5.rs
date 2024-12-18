use crate::builder3::{BuilderField, FieldType};
use crate::helper::extract_struct_fields;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{DeriveInput, Visibility};

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
  let build_fn = make_build_fn(vis, input_ident, &builder_fields);

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

fn make_storage(fields: &[BuilderField]) -> TokenStream2 {
  fields
    .iter()
    .map(|field| {
      let ident = &field.ident;
      let storage = match &field.ty {
        FieldType::Plain(ty) | FieldType::Optional(ty) => quote!(Option::<#ty>),
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

fn make_setters(vis: &Visibility, fields: &[BuilderField]) -> TokenStream2 {
  fields
    .iter()
    .map(|field| {
      let ident = &field.ident;
      let ident_type = match &field.ty {
        FieldType::Plain(ty) => ty,
        FieldType::Optional(ty) => ty,
      };
      quote! {
        #vis fn #ident(&mut self, #ident: #ident_type) -> &mut Self {
          self.#ident = Some(#ident);
          self
        }
      }
    })
    .collect()
}

/// 此函数用于动态生成一个构建器的build方法，该方法负责将构建器的当前状态转换为所需类型的实例。
/// 它会检查所有必需的字段是否已设置，并将所有字段（包括可选字段）适当地赋值给新创建的实例
///
/// # Parameters
/// - `vis`: 指定生成函数的可见性，例如`pub`或私有
/// - `input_ident`: 要构建的对象的类型标识符
/// - `fields`: 包含构建器所有字段信息的向量，用于生成字段检查和赋值代码
///
/// # Returns
/// - 生成的build函数的令牌流，可以用于构建指定类型的实例
fn make_build_fn(vis: &Visibility, input_ident: &Ident, fields: &[BuilderField]) -> TokenStream2 {
  // 遍历所有字段，为每个非可选字段生成检查代码，确保它们在构建之前已被设置
  let required_field_checks = fields.iter().filter_map(|field| {
    let ident = &field.ident;
    match &field.ty {
      FieldType::Plain(_) => Some(quote! {
        let #ident = self.#ident.take().ok_or_else(|| {
          Box::<dyn core::error::Error>::from(format!("value is not set: {}", stringify!(#ident)))
        })?;
      }),
      FieldType::Optional(_) => None,
    }
  });

  // 为所有字段生成赋值代码，将构建器的字段值赋给新创建的对象
  let field_assignment = fields.iter().map(|field| {
    let ident = &field.ident;
    let expr = match &field.ty {
      FieldType::Plain(_) => quote!(#ident),
      FieldType::Optional(_) => quote!(self.#ident.take()),
    };
    quote! {
      #ident: #expr,
    }
  });

  // 生成并返回完整的build函数的令牌流
  quote! {
    #vis fn build(&mut self) -> Result<#input_ident, Box<dyn core::error::Error>> {
      #(#required_field_checks)*

      Ok(#input_ident {
        #(#field_assignment)*
      })
    }
  }
}

