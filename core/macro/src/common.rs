// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

//! Consts and type-introspection helpers shared by more than one derive
//! macro in this crate.

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    BinOp, Error, Expr, ExprBinary, Field, GenericArgument, Ident, PathArguments, PathSegment, Result as SynResult,
    Type,
};

pub(crate) const PRIMITIVES: &[&str] = &[
    "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize", "bool", "f32", "f64",
];

pub(crate) const ABI_ENUMS: &[&str] = &[
    "FileDiffKind",
    "PackageDiffKind",
    "DiffFileSource",
    "FsKind",
    "PartitionKind",
    "InitramfsGenerator",
    "ErrorDomain",
    "ErrorKind",
    "TriggerPosition",
];

pub(crate) const VALIDATABLE_LIB_ENTRYS_COMPOSITES: &[&str] = &[
    "CDiffFileCommonEntry",
    "CDiffPrefixFileEntry",
    "CDiffConfigFileEntry",
    "CDiffUntrackedFileEntry",
    "CDiffPackageEntry",
    "CConfigCommitEntry",
    "CSearchFileEntry",
    "CPrefixEntry",
    "CHistoryEntry",
];

pub(crate) const VALIDATABLE_LIB_PACKAGE_COMPOSITES: &[&str] = &[
    "CVersion",
    "CPackageMeta",
    "CPackageInfo",
    "CPackageDependency",
    "CPackageTrigger",
];

pub(crate) const VALIDATABLE_LIB_REQUEST_COMPOSITES: &[&str] = &["CRequestBase", "CFileTransfer"];

pub(crate) const VALIDATABLE_COMPOSITE_CATEGORIES: &[&[&str]] = &[
    VALIDATABLE_LIB_ENTRYS_COMPOSITES,
    VALIDATABLE_LIB_PACKAGE_COMPOSITES,
    VALIDATABLE_LIB_REQUEST_COMPOSITES,
];

pub(crate) fn is_validatable_composite(name: &str) -> bool {
    VALIDATABLE_COMPOSITE_CATEGORIES
        .iter()
        .any(|category| category.contains(&name))
}

pub(crate) fn generic_arg(segment: &PathSegment) -> Option<&Type> {
    let PathArguments::AngleBracketed(args) = &segment.arguments else {
        return None;
    };

    args.args.iter().find_map(|arg| match arg {
        GenericArgument::Type(ty) => Some(ty),
        _ => None,
    })
}

pub(crate) fn segment_name(ty: &Type) -> Option<String> {
    let Type::Path(type_path) = ty else {
        return None;
    };

    type_path.path.segments.last().map(|segment| segment.ident.to_string())
}

pub(crate) fn is_str_type(ty: &Type) -> bool {
    matches!(ty, Type::Path(path) if path.path.is_ident("str"))
}

pub(crate) struct FieldCondition {
    pub field: Ident,
    pub value: Expr,
}

impl FieldCondition {
    pub(crate) fn on(&self, receiver: TokenStream2) -> TokenStream2 {
        let FieldCondition { field, value } = self;
        quote! { #receiver.#field == #value }
    }
}

pub(crate) fn field_condition(field: &Field, attr_name: &str) -> Option<SynResult<FieldCondition>> {
    let attr = field.attrs.iter().find(|attr| attr.path().is_ident(attr_name))?;

    Some(attr.parse_args::<ExprBinary>().and_then(|binary| {
        if !matches!(binary.op, BinOp::Eq(_)) {
            return Err(Error::new_spanned(binary.op, "expected `field == value`"));
        }

        let condition_field = match binary.left.as_ref() {
            Expr::Path(path) => path.path.get_ident().cloned(),
            _ => None,
        };
        let Some(condition_field) = condition_field else {
            return Err(Error::new_spanned(
                &binary.left,
                "expected a field name on the left of `==`",
            ));
        };

        Ok(FieldCondition {
            field: condition_field,
            value: *binary.right,
        })
    }))
}
