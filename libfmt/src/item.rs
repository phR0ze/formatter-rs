use crate::attrs;
use crate::iter::IterDelimited;
use crate::path::PathKind;
use crate::{engine::Engine, DUMMY};
use proc_macro2::TokenStream;
use syn::{
    Fields, FnArg, ForeignItem, ForeignItemFn, ForeignItemMacro, ForeignItemStatic,
    ForeignItemType, ImplItem, ImplItemConst, ImplItemFn, ImplItemMacro, ImplItemType, Item,
    ItemConst, ItemEnum, ItemExternCrate, ItemFn, ItemForeignMod, ItemImpl, ItemMacro, ItemMod,
    ItemStatic, ItemStruct, ItemTrait, ItemTraitAlias, ItemType, ItemUnion, ItemUse, Receiver,
    Signature, StaticMutability, TraitItem, TraitItemConst, TraitItemFn, TraitItemMacro,
    TraitItemType, Type, UseGlob, UseGroup, UseName, UsePath, UseRename, UseTree, Variadic,
};

impl Engine {
    pub fn item(&mut self, item: &Item) {
        match item {
            #![cfg_attr(all(test, exhaustive), deny(non_exhaustive_omitted_patterns))]
            Item::Const(item) => self.item_const(item),
            Item::Enum(item) => self.item_enum(item),
            Item::ExternCrate(item) => self.item_extern_crate(item),
            Item::Fn(item) => self.item_fn(item),
            Item::ForeignMod(item) => self.item_foreign_mod(item),
            Item::Impl(item) => self.item_impl(item),
            Item::Macro(item) => self.item_macro(item),
            Item::Mod(item) => self.item_mod(item),
            Item::Static(item) => self.item_static(item),
            Item::Struct(item) => self.item_struct(item),
            Item::Trait(item) => self.item_trait(item),
            Item::TraitAlias(item) => self.item_trait_alias(item),
            Item::Type(item) => self.item_type(item),
            Item::Union(item) => self.item_union(item),
            Item::Use(item) => self.item_use(item),
            Item::Verbatim(item) => self.item_verbatim(item),
            _ => unimplemented!("unknown Item"),
        }
    }

    fn item_const(&mut self, item: &ItemConst) {
        self.outer_attrs(&item.attrs);
        self.scan_begin_consistent(0);
        self.visibility(&item.vis);
        self.scan_string("const ");
        self.ident(&item.ident);
        self.generics(&item.generics);
        self.scan_string(": ");
        self.ty(&item.ty);
        self.scan_string(" = ");
        self.neverbreak();
        self.expr(&item.expr);
        self.scan_string(";");
        self.scan_end();
        self.scan_hardbreak();
    }

    fn item_enum(&mut self, item: &ItemEnum) {
        self.outer_attrs(&item.attrs);
        self.scan_begin_consistent(self.config.indent);
        self.visibility(&item.vis);
        self.scan_string("enum ");
        self.ident(&item.ident);
        self.generics(&item.generics);
        self.where_clause_for_body(&item.generics.where_clause);
        self.scan_string("{");
        self.hardbreak_if_nonempty();
        for variant in &item.variants {
            self.variant(variant);
            self.scan_string(",");
            self.scan_hardbreak();
        }
        self.offset(-self.config.indent);
        self.scan_end();
        self.scan_string("}");
        self.scan_hardbreak();
    }

    fn item_extern_crate(&mut self, item: &ItemExternCrate) {
        self.outer_attrs(&item.attrs);
        self.visibility(&item.vis);
        self.scan_string("extern crate ");
        self.ident(&item.ident);
        if let Some((_as_token, rename)) = &item.rename {
            self.scan_string(" as ");
            self.ident(rename);
        }
        self.scan_string(";");
        self.scan_hardbreak();
    }

    fn item_fn(&mut self, item: &ItemFn) {
        self.outer_attrs(&item.attrs);
        self.scan_begin_consistent(self.config.indent);
        self.visibility(&item.vis);
        self.signature(&item.sig);
        self.where_clause_for_body(&item.sig.generics.where_clause);
        self.scan_string("{");
        self.hardbreak_if_nonempty();
        self.inner_attrs(&item.attrs);
        for stmt in &item.block.stmts {
            self.stmt(stmt);
        }
        self.offset(-self.config.indent);
        self.scan_end();
        self.scan_string("}");
        self.scan_hardbreak();
    }

    fn item_foreign_mod(&mut self, item: &ItemForeignMod) {
        self.outer_attrs(&item.attrs);
        self.scan_begin_consistent(self.config.indent);
        if item.unsafety.is_some() {
            self.scan_string("unsafe ");
        }
        self.abi(&item.abi);
        self.scan_string("{");
        self.hardbreak_if_nonempty();
        self.inner_attrs(&item.attrs);
        for foreign_item in &item.items {
            self.foreign_item(foreign_item);
        }
        self.offset(-self.config.indent);
        self.scan_end();
        self.scan_string("}");
        self.scan_hardbreak();
    }

    fn item_impl(&mut self, item: &ItemImpl) {
        self.outer_attrs(&item.attrs);
        self.scan_begin_consistent(self.config.indent);
        self.scan_begin_inconsistent(-self.config.indent);
        self.scan_begin_consistent(self.config.indent);
        if item.defaultness.is_some() {
            self.scan_string("default ");
        }
        if item.unsafety.is_some() {
            self.scan_string("unsafe ");
        }
        self.scan_string("impl");
        self.generics(&item.generics);
        self.scan_end();
        self.nbsp();
        if let Some((negative_polarity, path, _for_token)) = &item.trait_ {
            if negative_polarity.is_some() {
                self.scan_string("!");
            }
            self.path(path, PathKind::Type);
            self.scan_space();
            self.scan_string("for ");
        }
        self.ty(&item.self_ty);
        self.scan_end();
        self.where_clause_for_body(&item.generics.where_clause);
        self.scan_string("{");
        self.hardbreak_if_nonempty();
        self.inner_attrs(&item.attrs);
        for impl_item in &item.items {
            self.impl_item(impl_item);
        }
        self.offset(-self.config.indent);
        self.scan_end();
        self.scan_string("}");
        self.scan_hardbreak();
    }

    fn item_macro(&mut self, item: &ItemMacro) {
        self.outer_attrs(&item.attrs);
        let semicolon = true;
        self.mac(&item.mac, item.ident.as_ref(), semicolon);
        self.scan_hardbreak();
    }

    fn item_mod(&mut self, item: &ItemMod) {
        self.outer_attrs(&item.attrs);
        self.scan_begin_consistent(self.config.indent);
        self.visibility(&item.vis);
        if item.unsafety.is_some() {
            self.scan_string("unsafe ");
        }
        self.scan_string("mod ");
        self.ident(&item.ident);
        if let Some((_brace, items)) = &item.content {
            self.scan_string(" {");
            self.hardbreak_if_nonempty();
            self.inner_attrs(&item.attrs);
            for item in items {
                self.item(item);
            }
            self.offset(-self.config.indent);
            self.scan_end();
            self.scan_string("}");
        } else {
            self.scan_string(";");
            self.scan_end();
        }
        self.scan_hardbreak();
    }

    fn item_static(&mut self, item: &ItemStatic) {
        self.outer_attrs(&item.attrs);
        self.scan_begin_consistent(0);
        self.visibility(&item.vis);
        self.scan_string("static ");
        self.static_mutability(&item.mutability);
        self.ident(&item.ident);
        self.scan_string(": ");
        self.ty(&item.ty);
        self.scan_string(" = ");
        self.neverbreak();
        self.expr(&item.expr);
        self.scan_string(";");
        self.scan_end();
        self.scan_hardbreak();
    }

    fn item_struct(&mut self, item: &ItemStruct) {
        // Print out any outer attributes e.g. comments
        self.outer_attrs(&item.attrs);

        // Don't print DUMMY value that was used to trick syn for comments
        if item.ident == DUMMY {
            return;
        }

        self.scan_begin_consistent(self.config.indent);
        self.visibility(&item.vis);
        self.scan_string("struct ");
        self.ident(&item.ident);
        self.generics(&item.generics);
        match &item.fields {
            Fields::Named(fields) => {
                self.where_clause_for_body(&item.generics.where_clause);
                self.scan_string("{");
                self.hardbreak_if_nonempty();

                // Look ahead to print trailing comments
                let mut iter = fields.named.iter().peekable();
                while let Some(field) = iter.next() {
                    // Don't print DUMMY fields used for trailing comments
                    if let Some(ident) = field.ident.as_ref() {
                        if ident == DUMMY {
                            continue;
                        }
                    }

                    // Scan the field portion
                    self.scan_field(field);
                    self.scan_string(",");

                    // Scan any trailing comment from the next field's outer attributes
                    let trailing = match iter.peek() {
                        Some(next) => self.scan_trailing_comment(&next.attrs),
                        None => false,
                    };
                    if !trailing {
                        self.scan_hardbreak();
                    }
                }
                self.offset(-self.config.indent);
                self.scan_end();
                self.scan_string("}");
            }
            Fields::Unnamed(fields) => {
                self.fields_unnamed(fields);
                self.where_clause_semi(&item.generics.where_clause);
                self.scan_end();
            }
            Fields::Unit => {
                self.where_clause_semi(&item.generics.where_clause);
                self.scan_end();
            }
        }
        self.scan_hardbreak();
    }

    fn item_trait(&mut self, item: &ItemTrait) {
        self.outer_attrs(&item.attrs);
        self.scan_begin_consistent(self.config.indent);
        self.visibility(&item.vis);
        if item.unsafety.is_some() {
            self.scan_string("unsafe ");
        }
        if item.auto_token.is_some() {
            self.scan_string("auto ");
        }
        self.scan_string("trait ");
        self.ident(&item.ident);
        self.generics(&item.generics);
        for supertrait in item.supertraits.iter().delimited() {
            if supertrait.is_first {
                self.scan_string(": ");
            } else {
                self.scan_string(" + ");
            }
            self.type_param_bound(&supertrait);
        }
        self.where_clause_for_body(&item.generics.where_clause);
        self.scan_string("{");
        self.hardbreak_if_nonempty();
        self.inner_attrs(&item.attrs);
        for trait_item in &item.items {
            self.trait_item(trait_item);
        }
        self.offset(-self.config.indent);
        self.scan_end();
        self.scan_string("}");
        self.scan_hardbreak();
    }

    fn item_trait_alias(&mut self, item: &ItemTraitAlias) {
        self.outer_attrs(&item.attrs);
        self.scan_begin_consistent(self.config.indent);
        self.visibility(&item.vis);
        self.scan_string("trait ");
        self.ident(&item.ident);
        self.generics(&item.generics);
        self.scan_string(" = ");
        self.neverbreak();
        for bound in item.bounds.iter().delimited() {
            if !bound.is_first {
                self.scan_space();
                self.scan_string("+ ");
            }
            self.type_param_bound(&bound);
        }
        self.where_clause_semi(&item.generics.where_clause);
        self.scan_end();
        self.scan_hardbreak();
    }

    fn item_type(&mut self, item: &ItemType) {
        self.outer_attrs(&item.attrs);
        self.scan_begin_consistent(self.config.indent);
        self.visibility(&item.vis);
        self.scan_string("type ");
        self.ident(&item.ident);
        self.generics(&item.generics);
        self.where_clause_oneline(&item.generics.where_clause);
        self.scan_string("= ");
        self.neverbreak();
        self.scan_begin_inconsistent(-self.config.indent);
        self.ty(&item.ty);
        self.scan_end();
        self.scan_string(";");
        self.scan_end();
        self.scan_hardbreak();
    }

    fn item_union(&mut self, item: &ItemUnion) {
        self.outer_attrs(&item.attrs);
        self.scan_begin_consistent(self.config.indent);
        self.visibility(&item.vis);
        self.scan_string("union ");
        self.ident(&item.ident);
        self.generics(&item.generics);
        self.where_clause_for_body(&item.generics.where_clause);
        self.scan_string("{");
        self.hardbreak_if_nonempty();
        for field in &item.fields.named {
            self.scan_field(field);
            self.scan_string(",");
            self.scan_hardbreak();
        }
        self.offset(-self.config.indent);
        self.scan_end();
        self.scan_string("}");
        self.scan_hardbreak();
    }

    fn item_use(&mut self, item: &ItemUse) {
        self.outer_attrs(&item.attrs);
        self.visibility(&item.vis);
        self.scan_string("use ");
        if item.leading_colon.is_some() {
            self.scan_string("::");
        }
        self.use_tree(&item.tree);
        self.scan_string(";");
        self.scan_hardbreak();
    }

    #[cfg(not(feature = "verbatim"))]
    fn item_verbatim(&mut self, item: &TokenStream) {
        if !item.is_empty() {
            unimplemented!("Item::Verbatim `{}`", item);
        }
        self.scan_hardbreak();
    }

    #[cfg(feature = "verbatim")]
    fn item_verbatim(&mut self, tokens: &TokenStream) {
        use syn::parse::{Parse, ParseStream, Result};
        use syn::punctuated::Punctuated;
        use syn::{
            braced, parenthesized, token, Attribute, Generics, Ident, Lifetime, Token, Visibility,
        };
        use verbatim::{
            FlexibleItemConst, FlexibleItemFn, FlexibleItemStatic, FlexibleItemType,
            WhereClauseLocation,
        };

        enum ItemVerbatim {
            Empty,
            Ellipsis,
            ConstFlexible(FlexibleItemConst),
            FnFlexible(FlexibleItemFn),
            ImplFlexible(ImplFlexible),
            Macro2(Macro2),
            StaticFlexible(FlexibleItemStatic),
            TypeFlexible(FlexibleItemType),
            UseBrace(UseBrace),
        }

        struct ImplFlexible {
            attrs: Vec<Attribute>,
            vis: Visibility,
            defaultness: bool,
            unsafety: bool,
            generics: Generics,
            constness: ImplConstness,
            negative_impl: bool,
            trait_: Option<Type>,
            self_ty: Type,
            items: Vec<ImplItem>,
        }

        enum ImplConstness {
            None,
            MaybeConst,
            Const,
        }

        struct Macro2 {
            attrs: Vec<Attribute>,
            vis: Visibility,
            ident: Ident,
            args: Option<TokenStream>,
            body: TokenStream,
        }

        struct UseBrace {
            attrs: Vec<Attribute>,
            vis: Visibility,
            trees: Punctuated<RootUseTree, Token![,]>,
        }

        struct RootUseTree {
            leading_colon: Option<Token![::]>,
            inner: UseTree,
        }

        impl Parse for ImplConstness {
            fn parse(input: ParseStream) -> Result<Self> {
                if input.parse::<Option<Token![?]>>()?.is_some() {
                    input.parse::<Token![const]>()?;
                    Ok(ImplConstness::MaybeConst)
                } else if input.parse::<Option<Token![const]>>()?.is_some() {
                    Ok(ImplConstness::Const)
                } else {
                    Ok(ImplConstness::None)
                }
            }
        }

        impl Parse for RootUseTree {
            fn parse(input: ParseStream) -> Result<Self> {
                Ok(RootUseTree {
                    leading_colon: input.parse()?,
                    inner: input.parse()?,
                })
            }
        }

        impl Parse for ItemVerbatim {
            fn parse(input: ParseStream) -> Result<Self> {
                if input.is_empty() {
                    return Ok(ItemVerbatim::Empty);
                } else if input.peek(Token![...]) {
                    input.parse::<Token![...]>()?;
                    return Ok(ItemVerbatim::Ellipsis);
                }

                let mut attrs = input.call(Attribute::parse_outer)?;
                let vis: Visibility = input.parse()?;

                let lookahead = input.lookahead1();
                if lookahead.peek(Token![const]) && (input.peek2(Ident) || input.peek2(Token![_])) {
                    let defaultness = false;
                    let flexible_item = FlexibleItemConst::parse(attrs, vis, defaultness, input)?;
                    Ok(ItemVerbatim::ConstFlexible(flexible_item))
                } else if input.peek(Token![const])
                    || lookahead.peek(Token![async])
                    || lookahead.peek(Token![unsafe]) && !input.peek2(Token![impl])
                    || lookahead.peek(Token![extern])
                    || lookahead.peek(Token![fn])
                {
                    let defaultness = false;
                    let flexible_item = FlexibleItemFn::parse(attrs, vis, defaultness, input)?;
                    Ok(ItemVerbatim::FnFlexible(flexible_item))
                } else if lookahead.peek(Token![default])
                    || input.peek(Token![unsafe])
                    || lookahead.peek(Token![impl])
                {
                    let defaultness = input.parse::<Option<Token![default]>>()?.is_some();
                    let unsafety = input.parse::<Option<Token![unsafe]>>()?.is_some();
                    input.parse::<Token![impl]>()?;
                    let has_generics = input.peek(Token![<])
                        && (input.peek2(Token![>])
                            || input.peek2(Token![#])
                            || (input.peek2(Ident) || input.peek2(Lifetime))
                                && (input.peek3(Token![:])
                                    || input.peek3(Token![,])
                                    || input.peek3(Token![>])
                                    || input.peek3(Token![=]))
                            || input.peek2(Token![const]));
                    let mut generics: Generics = if has_generics {
                        input.parse()?
                    } else {
                        Generics::default()
                    };
                    let constness: ImplConstness = input.parse()?;
                    let negative_impl =
                        !input.peek2(token::Brace) && input.parse::<Option<Token![!]>>()?.is_some();
                    let first_ty: Type = input.parse()?;
                    let (trait_, self_ty) = if input.parse::<Option<Token![for]>>()?.is_some() {
                        (Some(first_ty), input.parse()?)
                    } else {
                        (None, first_ty)
                    };
                    generics.where_clause = input.parse()?;
                    let content;
                    braced!(content in input);
                    let inner_attrs = content.call(Attribute::parse_inner)?;
                    attrs.extend(inner_attrs);
                    let mut items = Vec::new();
                    while !content.is_empty() {
                        items.push(content.parse()?);
                    }
                    Ok(ItemVerbatim::ImplFlexible(ImplFlexible {
                        attrs,
                        vis,
                        defaultness,
                        unsafety,
                        generics,
                        constness,
                        negative_impl,
                        trait_,
                        self_ty,
                        items,
                    }))
                } else if lookahead.peek(Token![macro]) {
                    input.parse::<Token![macro]>()?;
                    let ident: Ident = input.parse()?;
                    let args = if input.peek(token::Paren) {
                        let paren_content;
                        parenthesized!(paren_content in input);
                        Some(paren_content.parse::<TokenStream>()?)
                    } else {
                        None
                    };
                    let brace_content;
                    braced!(brace_content in input);
                    let body: TokenStream = brace_content.parse()?;
                    Ok(ItemVerbatim::Macro2(Macro2 {
                        attrs,
                        vis,
                        ident,
                        args,
                        body,
                    }))
                } else if lookahead.peek(Token![static]) {
                    let flexible_item = FlexibleItemStatic::parse(attrs, vis, input)?;
                    Ok(ItemVerbatim::StaticFlexible(flexible_item))
                } else if lookahead.peek(Token![type]) {
                    let defaultness = false;
                    let flexible_item = FlexibleItemType::parse(
                        attrs,
                        vis,
                        defaultness,
                        input,
                        WhereClauseLocation::BeforeEq,
                    )?;
                    Ok(ItemVerbatim::TypeFlexible(flexible_item))
                } else if lookahead.peek(Token![use]) {
                    input.parse::<Token![use]>()?;
                    let content;
                    braced!(content in input);
                    let trees = content.parse_terminated(RootUseTree::parse, Token![,])?;
                    input.parse::<Token![;]>()?;
                    Ok(ItemVerbatim::UseBrace(UseBrace { attrs, vis, trees }))
                } else {
                    Err(lookahead.error())
                }
            }
        }

        let item: ItemVerbatim = match syn::parse2(tokens.clone()) {
            Ok(item) => item,
            Err(_) => unimplemented!("Item::Verbatim `{}`", tokens),
        };

        match item {
            ItemVerbatim::Empty => {
                self.hardbreak();
            }
            ItemVerbatim::Ellipsis => {
                self.scan_string("...");
                self.hardbreak();
            }
            ItemVerbatim::ConstFlexible(item) => {
                self.flexible_item_const(&item);
            }
            ItemVerbatim::FnFlexible(item) => {
                self.flexible_item_fn(&item);
            }
            ItemVerbatim::ImplFlexible(item) => {
                self.outer_attrs(&item.attrs);
                self.scan_begin_consistent(self.config.indent);
                self.scan_begin_inconsistent(-self.config.indent);
                self.scan_begin_consistent(self.config.indent);
                self.visibility(&item.vis);
                if item.defaultness {
                    self.scan_string("default ");
                }
                if item.unsafety {
                    self.scan_string("unsafe ");
                }
                self.scan_string("impl");
                self.generics(&item.generics);
                self.scan_end();
                self.nbsp();
                match item.constness {
                    ImplConstness::None => {}
                    ImplConstness::MaybeConst => self.scan_string("?const "),
                    ImplConstness::Const => self.scan_string("const "),
                }
                if item.negative_impl {
                    self.scan_string("!");
                }
                if let Some(trait_) = &item.trait_ {
                    self.ty(trait_);
                    self.space();
                    self.scan_string("for ");
                }
                self.ty(&item.self_ty);
                self.scan_end();
                self.where_clause_for_body(&item.generics.where_clause);
                self.scan_string("{");
                self.hardbreak_if_nonempty();
                self.inner_attrs(&item.attrs);
                for impl_item in &item.items {
                    self.impl_item(impl_item);
                }
                self.offset(-self.config.indent);
                self.scan_end();
                self.scan_string("}");
                self.hardbreak();
            }
            ItemVerbatim::Macro2(item) => {
                self.outer_attrs(&item.attrs);
                self.visibility(&item.vis);
                self.scan_string("macro ");
                self.ident(&item.ident);
                if let Some(args) = &item.args {
                    self.scan_string("(");
                    self.scan_begin_consistent(self.config.indent);
                    self.zerobreak();
                    self.scan_begin_inconsistent(0);
                    self.macro_rules_tokens(args.clone(), true);
                    self.scan_end();
                    self.zerobreak();
                    self.offset(-self.config.indent);
                    self.scan_end();
                    self.scan_string(")");
                }
                self.scan_string(" {");
                if !item.body.is_empty() {
                    self.neverbreak();
                    self.scan_begin_consistent(self.config.indent);
                    self.hardbreak();
                    self.scan_begin_inconsistent(0);
                    self.macro_rules_tokens(item.body.clone(), false);
                    self.scan_end();
                    self.hardbreak();
                    self.offset(-self.config.indent);
                    self.scan_end();
                }
                self.scan_string("}");
                self.hardbreak();
            }
            ItemVerbatim::StaticFlexible(item) => {
                self.flexible_item_static(&item);
            }
            ItemVerbatim::TypeFlexible(item) => {
                self.flexible_item_type(&item);
            }
            ItemVerbatim::UseBrace(item) => {
                self.outer_attrs(&item.attrs);
                self.visibility(&item.vis);
                self.scan_string("use ");
                if item.trees.len() == 1 {
                    self.scan_string("::");
                    self.use_tree(&item.trees[0].inner);
                } else {
                    self.scan_begin_consistent(self.config.indent);
                    self.scan_string("{");
                    self.zerobreak();
                    self.scan_begin_inconsistent(0);
                    for use_tree in item.trees.iter().delimited() {
                        if use_tree.leading_colon.is_some() {
                            self.scan_string("::");
                        }
                        self.use_tree(&use_tree.inner);
                        if !use_tree.is_last {
                            self.scan_string(",");
                            let mut use_tree = &use_tree.inner;
                            while let UseTree::Path(use_path) = use_tree {
                                use_tree = &use_path.tree;
                            }
                            if let UseTree::Group(_) = use_tree {
                                self.hardbreak();
                            } else {
                                self.space();
                            }
                        }
                    }
                    self.scan_end();
                    self.trailing_comma(true);
                    self.offset(-self.config.indent);
                    self.scan_string("}");
                    self.scan_end();
                }
                self.scan_string(";");
                self.hardbreak();
            }
        }
    }

    fn use_tree(&mut self, use_tree: &UseTree) {
        match use_tree {
            UseTree::Path(use_path) => self.use_path(use_path),
            UseTree::Name(use_name) => self.use_name(use_name),
            UseTree::Rename(use_rename) => self.use_rename(use_rename),
            UseTree::Glob(use_glob) => self.use_glob(use_glob),
            UseTree::Group(use_group) => self.use_group(use_group),
        }
    }

    fn use_path(&mut self, use_path: &UsePath) {
        self.ident(&use_path.ident);
        self.scan_string("::");
        self.use_tree(&use_path.tree);
    }

    fn use_name(&mut self, use_name: &UseName) {
        self.ident(&use_name.ident);
    }

    fn use_rename(&mut self, use_rename: &UseRename) {
        self.ident(&use_rename.ident);
        self.scan_string(" as ");
        self.ident(&use_rename.rename);
    }

    fn use_glob(&mut self, use_glob: &UseGlob) {
        let _ = use_glob;
        self.scan_string("*");
    }

    fn use_group(&mut self, use_group: &UseGroup) {
        if use_group.items.is_empty() {
            self.scan_string("{}");
        } else if use_group.items.len() == 1
            && match &use_group.items[0] {
                UseTree::Rename(use_rename) => use_rename.ident != "self",
                _ => true,
            }
        {
            self.use_tree(&use_group.items[0]);
        } else {
            self.scan_begin_consistent(self.config.indent);
            self.scan_string("{");
            self.zerobreak();
            self.scan_begin_inconsistent(0);
            for use_tree in use_group.items.iter().delimited() {
                self.use_tree(&use_tree);
                if !use_tree.is_last {
                    self.scan_string(",");
                    let mut use_tree = *use_tree;
                    while let UseTree::Path(use_path) = use_tree {
                        use_tree = &use_path.tree;
                    }
                    if let UseTree::Group(_) = use_tree {
                        self.scan_hardbreak();
                    } else {
                        self.scan_space();
                    }
                }
            }
            self.scan_end();
            self.trailing_comma(true);
            self.offset(-self.config.indent);
            self.scan_string("}");
            self.scan_end();
        }
    }

    fn foreign_item(&mut self, foreign_item: &ForeignItem) {
        match foreign_item {
            #![cfg_attr(all(test, exhaustive), deny(non_exhaustive_omitted_patterns))]
            ForeignItem::Fn(item) => self.foreign_item_fn(item),
            ForeignItem::Static(item) => self.foreign_item_static(item),
            ForeignItem::Type(item) => self.foreign_item_type(item),
            ForeignItem::Macro(item) => self.foreign_item_macro(item),
            ForeignItem::Verbatim(item) => self.foreign_item_verbatim(item),
            _ => unimplemented!("unknown ForeignItem"),
        }
    }

    fn foreign_item_fn(&mut self, foreign_item: &ForeignItemFn) {
        self.outer_attrs(&foreign_item.attrs);
        self.scan_begin_consistent(self.config.indent);
        self.visibility(&foreign_item.vis);
        self.signature(&foreign_item.sig);
        self.where_clause_semi(&foreign_item.sig.generics.where_clause);
        self.scan_end();
        self.scan_hardbreak();
    }

    fn foreign_item_static(&mut self, foreign_item: &ForeignItemStatic) {
        self.outer_attrs(&foreign_item.attrs);
        self.scan_begin_consistent(0);
        self.visibility(&foreign_item.vis);
        self.scan_string("static ");
        self.static_mutability(&foreign_item.mutability);
        self.ident(&foreign_item.ident);
        self.scan_string(": ");
        self.ty(&foreign_item.ty);
        self.scan_string(";");
        self.scan_end();
        self.scan_hardbreak();
    }

    fn foreign_item_type(&mut self, foreign_item: &ForeignItemType) {
        self.outer_attrs(&foreign_item.attrs);
        self.scan_begin_consistent(0);
        self.visibility(&foreign_item.vis);
        self.scan_string("type ");
        self.ident(&foreign_item.ident);
        self.generics(&foreign_item.generics);
        self.scan_string(";");
        self.scan_end();
        self.scan_hardbreak();
    }

    fn foreign_item_macro(&mut self, foreign_item: &ForeignItemMacro) {
        self.outer_attrs(&foreign_item.attrs);
        let semicolon = true;
        self.mac(&foreign_item.mac, None, semicolon);
        self.scan_hardbreak();
    }

    #[cfg(not(feature = "verbatim"))]
    fn foreign_item_verbatim(&mut self, foreign_item: &TokenStream) {
        if !foreign_item.is_empty() {
            unimplemented!("ForeignItem::Verbatim `{}`", foreign_item);
        }
        self.scan_hardbreak();
    }

    #[cfg(feature = "verbatim")]
    fn foreign_item_verbatim(&mut self, tokens: &TokenStream) {
        use syn::parse::{Parse, ParseStream, Result};
        use syn::{Attribute, Token, Visibility};
        use verbatim::{FlexibleItemFn, FlexibleItemStatic, FlexibleItemType, WhereClauseLocation};

        enum ForeignItemVerbatim {
            Empty,
            Ellipsis,
            FnFlexible(FlexibleItemFn),
            StaticFlexible(FlexibleItemStatic),
            TypeFlexible(FlexibleItemType),
        }

        impl Parse for ForeignItemVerbatim {
            fn parse(input: ParseStream) -> Result<Self> {
                if input.is_empty() {
                    return Ok(ForeignItemVerbatim::Empty);
                } else if input.peek(Token![...]) {
                    input.parse::<Token![...]>()?;
                    return Ok(ForeignItemVerbatim::Ellipsis);
                }

                let attrs = input.call(Attribute::parse_outer)?;
                let vis: Visibility = input.parse()?;
                let defaultness = false;

                let lookahead = input.lookahead1();
                if lookahead.peek(Token![const])
                    || lookahead.peek(Token![async])
                    || lookahead.peek(Token![unsafe])
                    || lookahead.peek(Token![extern])
                    || lookahead.peek(Token![fn])
                {
                    let flexible_item = FlexibleItemFn::parse(attrs, vis, defaultness, input)?;
                    Ok(ForeignItemVerbatim::FnFlexible(flexible_item))
                } else if lookahead.peek(Token![static]) {
                    let flexible_item = FlexibleItemStatic::parse(attrs, vis, input)?;
                    Ok(ForeignItemVerbatim::StaticFlexible(flexible_item))
                } else if lookahead.peek(Token![type]) {
                    let flexible_item = FlexibleItemType::parse(
                        attrs,
                        vis,
                        defaultness,
                        input,
                        WhereClauseLocation::Both,
                    )?;
                    Ok(ForeignItemVerbatim::TypeFlexible(flexible_item))
                } else {
                    Err(lookahead.error())
                }
            }
        }

        let foreign_item: ForeignItemVerbatim = match syn::parse2(tokens.clone()) {
            Ok(foreign_item) => foreign_item,
            Err(_) => unimplemented!("ForeignItem::Verbatim `{}`", tokens),
        };

        match foreign_item {
            ForeignItemVerbatim::Empty => {
                self.hardbreak();
            }
            ForeignItemVerbatim::Ellipsis => {
                self.scan_string("...");
                self.hardbreak();
            }
            ForeignItemVerbatim::FnFlexible(foreign_item) => {
                self.flexible_item_fn(&foreign_item);
            }
            ForeignItemVerbatim::StaticFlexible(foreign_item) => {
                self.flexible_item_static(&foreign_item);
            }
            ForeignItemVerbatim::TypeFlexible(foreign_item) => {
                self.flexible_item_type(&foreign_item);
            }
        }
    }

    fn trait_item(&mut self, trait_item: &TraitItem) {
        match trait_item {
            #![cfg_attr(all(test, exhaustive), deny(non_exhaustive_omitted_patterns))]
            TraitItem::Const(item) => self.trait_item_const(item),
            TraitItem::Fn(item) => self.trait_item_fn(item),
            TraitItem::Type(item) => self.trait_item_type(item),
            TraitItem::Macro(item) => self.trait_item_macro(item),
            TraitItem::Verbatim(item) => self.trait_item_verbatim(item),
            _ => unimplemented!("unknown TraitItem"),
        }
    }

    fn trait_item_const(&mut self, trait_item: &TraitItemConst) {
        self.outer_attrs(&trait_item.attrs);
        self.scan_begin_consistent(0);
        self.scan_string("const ");
        self.ident(&trait_item.ident);
        self.generics(&trait_item.generics);
        self.scan_string(": ");
        self.ty(&trait_item.ty);
        if let Some((_eq_token, default)) = &trait_item.default {
            self.scan_string(" = ");
            self.neverbreak();
            self.expr(default);
        }
        self.scan_string(";");
        self.scan_end();
        self.scan_hardbreak();
    }

    fn trait_item_fn(&mut self, trait_item: &TraitItemFn) {
        self.outer_attrs(&trait_item.attrs);
        self.scan_begin_consistent(self.config.indent);
        self.signature(&trait_item.sig);
        if let Some(block) = &trait_item.default {
            self.where_clause_for_body(&trait_item.sig.generics.where_clause);
            self.scan_string("{");
            self.hardbreak_if_nonempty();
            self.inner_attrs(&trait_item.attrs);
            for stmt in &block.stmts {
                self.stmt(stmt);
            }
            self.offset(-self.config.indent);
            self.scan_end();
            self.scan_string("}");
        } else {
            self.where_clause_semi(&trait_item.sig.generics.where_clause);
            self.scan_end();
        }
        self.scan_hardbreak();
    }

    fn trait_item_type(&mut self, trait_item: &TraitItemType) {
        self.outer_attrs(&trait_item.attrs);
        self.scan_begin_consistent(self.config.indent);
        self.scan_string("type ");
        self.ident(&trait_item.ident);
        self.generics(&trait_item.generics);
        for bound in trait_item.bounds.iter().delimited() {
            if bound.is_first {
                self.scan_string(": ");
            } else {
                self.scan_space();
                self.scan_string("+ ");
            }
            self.type_param_bound(&bound);
        }
        if let Some((_eq_token, default)) = &trait_item.default {
            self.scan_string(" = ");
            self.neverbreak();
            self.scan_begin_inconsistent(-self.config.indent);
            self.ty(default);
            self.scan_end();
        }
        self.where_clause_oneline_semi(&trait_item.generics.where_clause);
        self.scan_end();
        self.scan_hardbreak();
    }

    fn trait_item_macro(&mut self, trait_item: &TraitItemMacro) {
        self.outer_attrs(&trait_item.attrs);
        let semicolon = true;
        self.mac(&trait_item.mac, None, semicolon);
        self.scan_hardbreak();
    }

    #[cfg(not(feature = "verbatim"))]
    fn trait_item_verbatim(&mut self, trait_item: &TokenStream) {
        if !trait_item.is_empty() {
            unimplemented!("TraitItem::Verbatim `{}`", trait_item);
        }
        self.scan_hardbreak();
    }

    #[cfg(feature = "verbatim")]
    fn trait_item_verbatim(&mut self, tokens: &TokenStream) {
        use syn::parse::{Parse, ParseStream, Result};
        use syn::{Attribute, Ident, Token, Visibility};
        use verbatim::{FlexibleItemConst, FlexibleItemType, WhereClauseLocation};

        enum TraitItemVerbatim {
            Empty,
            Ellipsis,
            ConstFlexible(FlexibleItemConst),
            TypeFlexible(FlexibleItemType),
            PubOrDefault(PubOrDefaultTraitItem),
        }

        struct PubOrDefaultTraitItem {
            attrs: Vec<Attribute>,
            vis: Visibility,
            defaultness: bool,
            trait_item: TraitItem,
        }

        impl Parse for TraitItemVerbatim {
            fn parse(input: ParseStream) -> Result<Self> {
                if input.is_empty() {
                    return Ok(TraitItemVerbatim::Empty);
                } else if input.peek(Token![...]) {
                    input.parse::<Token![...]>()?;
                    return Ok(TraitItemVerbatim::Ellipsis);
                }

                let attrs = input.call(Attribute::parse_outer)?;
                let vis: Visibility = input.parse()?;
                let defaultness = input.parse::<Option<Token![default]>>()?.is_some();

                let lookahead = input.lookahead1();
                if lookahead.peek(Token![const]) && (input.peek2(Ident) || input.peek2(Token![_])) {
                    let flexible_item = FlexibleItemConst::parse(attrs, vis, defaultness, input)?;
                    Ok(TraitItemVerbatim::ConstFlexible(flexible_item))
                } else if lookahead.peek(Token![type]) {
                    let flexible_item = FlexibleItemType::parse(
                        attrs,
                        vis,
                        defaultness,
                        input,
                        WhereClauseLocation::AfterEq,
                    )?;
                    Ok(TraitItemVerbatim::TypeFlexible(flexible_item))
                } else if (input.peek(Token![const])
                    || lookahead.peek(Token![async])
                    || lookahead.peek(Token![unsafe])
                    || lookahead.peek(Token![extern])
                    || lookahead.peek(Token![fn]))
                    && (!matches!(vis, Visibility::Inherited) || defaultness)
                {
                    Ok(TraitItemVerbatim::PubOrDefault(PubOrDefaultTraitItem {
                        attrs,
                        vis,
                        defaultness,
                        trait_item: input.parse()?,
                    }))
                } else {
                    Err(lookahead.error())
                }
            }
        }

        let impl_item: TraitItemVerbatim = match syn::parse2(tokens.clone()) {
            Ok(impl_item) => impl_item,
            Err(_) => unimplemented!("TraitItem::Verbatim `{}`", tokens),
        };

        match impl_item {
            TraitItemVerbatim::Empty => {
                self.hardbreak();
            }
            TraitItemVerbatim::Ellipsis => {
                self.scan_string("...");
                self.hardbreak();
            }
            TraitItemVerbatim::ConstFlexible(trait_item) => {
                self.flexible_item_const(&trait_item);
            }
            TraitItemVerbatim::TypeFlexible(trait_item) => {
                self.flexible_item_type(&trait_item);
            }
            TraitItemVerbatim::PubOrDefault(trait_item) => {
                self.outer_attrs(&trait_item.attrs);
                self.visibility(&trait_item.vis);
                if trait_item.defaultness {
                    self.scan_string("default ");
                }
                self.trait_item(&trait_item.trait_item);
            }
        }
    }

    fn impl_item(&mut self, impl_item: &ImplItem) {
        match impl_item {
            #![cfg_attr(all(test, exhaustive), deny(non_exhaustive_omitted_patterns))]
            ImplItem::Const(item) => self.impl_item_const(item),
            ImplItem::Fn(item) => self.impl_item_fn(item),
            ImplItem::Type(item) => self.impl_item_type(item),
            ImplItem::Macro(item) => self.impl_item_macro(item),
            ImplItem::Verbatim(item) => self.impl_item_verbatim(item),
            _ => unimplemented!("unknown ImplItem"),
        }
    }

    fn impl_item_const(&mut self, impl_item: &ImplItemConst) {
        self.outer_attrs(&impl_item.attrs);
        self.scan_begin_consistent(0);
        self.visibility(&impl_item.vis);
        if impl_item.defaultness.is_some() {
            self.scan_string("default ");
        }
        self.scan_string("const ");
        self.ident(&impl_item.ident);
        self.generics(&impl_item.generics);
        self.scan_string(": ");
        self.ty(&impl_item.ty);
        self.scan_string(" = ");
        self.neverbreak();
        self.expr(&impl_item.expr);
        self.scan_string(";");
        self.scan_end();
        self.scan_hardbreak();
    }

    fn impl_item_fn(&mut self, impl_item: &ImplItemFn) {
        self.outer_attrs(&impl_item.attrs);
        self.scan_begin_consistent(self.config.indent);
        self.visibility(&impl_item.vis);
        if impl_item.defaultness.is_some() {
            self.scan_string("default ");
        }
        self.signature(&impl_item.sig);
        self.where_clause_for_body(&impl_item.sig.generics.where_clause);
        self.scan_string("{");
        self.hardbreak_if_nonempty();
        self.inner_attrs(&impl_item.attrs);
        for stmt in &impl_item.block.stmts {
            self.stmt(stmt);
        }
        self.offset(-self.config.indent);
        self.scan_end();
        self.scan_string("}");
        self.scan_hardbreak();
    }

    fn impl_item_type(&mut self, impl_item: &ImplItemType) {
        self.outer_attrs(&impl_item.attrs);
        self.scan_begin_consistent(self.config.indent);
        self.visibility(&impl_item.vis);
        if impl_item.defaultness.is_some() {
            self.scan_string("default ");
        }
        self.scan_string("type ");
        self.ident(&impl_item.ident);
        self.generics(&impl_item.generics);
        self.scan_string(" = ");
        self.neverbreak();
        self.scan_begin_inconsistent(-self.config.indent);
        self.ty(&impl_item.ty);
        self.scan_end();
        self.where_clause_oneline_semi(&impl_item.generics.where_clause);
        self.scan_end();
        self.scan_hardbreak();
    }

    fn impl_item_macro(&mut self, impl_item: &ImplItemMacro) {
        self.outer_attrs(&impl_item.attrs);
        let semicolon = true;
        self.mac(&impl_item.mac, None, semicolon);
        self.scan_hardbreak();
    }

    #[cfg(not(feature = "verbatim"))]
    fn impl_item_verbatim(&mut self, impl_item: &TokenStream) {
        if !impl_item.is_empty() {
            unimplemented!("ImplItem::Verbatim `{}`", impl_item);
        }
        self.scan_hardbreak();
    }

    #[cfg(feature = "verbatim")]
    fn impl_item_verbatim(&mut self, tokens: &TokenStream) {
        use syn::parse::{Parse, ParseStream, Result};
        use syn::{Attribute, Ident, Token, Visibility};
        use verbatim::{FlexibleItemConst, FlexibleItemFn, FlexibleItemType, WhereClauseLocation};

        enum ImplItemVerbatim {
            Empty,
            Ellipsis,
            ConstFlexible(FlexibleItemConst),
            FnFlexible(FlexibleItemFn),
            TypeFlexible(FlexibleItemType),
        }

        impl Parse for ImplItemVerbatim {
            fn parse(input: ParseStream) -> Result<Self> {
                if input.is_empty() {
                    return Ok(ImplItemVerbatim::Empty);
                } else if input.peek(Token![...]) {
                    input.parse::<Token![...]>()?;
                    return Ok(ImplItemVerbatim::Ellipsis);
                }

                let attrs = input.call(Attribute::parse_outer)?;
                let vis: Visibility = input.parse()?;
                let defaultness = input.parse::<Option<Token![default]>>()?.is_some();

                let lookahead = input.lookahead1();
                if lookahead.peek(Token![const]) && (input.peek2(Ident) || input.peek2(Token![_])) {
                    let flexible_item = FlexibleItemConst::parse(attrs, vis, defaultness, input)?;
                    Ok(ImplItemVerbatim::ConstFlexible(flexible_item))
                } else if input.peek(Token![const])
                    || lookahead.peek(Token![async])
                    || lookahead.peek(Token![unsafe])
                    || lookahead.peek(Token![extern])
                    || lookahead.peek(Token![fn])
                {
                    let flexible_item = FlexibleItemFn::parse(attrs, vis, defaultness, input)?;
                    Ok(ImplItemVerbatim::FnFlexible(flexible_item))
                } else if lookahead.peek(Token![type]) {
                    let flexible_item = FlexibleItemType::parse(
                        attrs,
                        vis,
                        defaultness,
                        input,
                        WhereClauseLocation::AfterEq,
                    )?;
                    Ok(ImplItemVerbatim::TypeFlexible(flexible_item))
                } else {
                    Err(lookahead.error())
                }
            }
        }

        let impl_item: ImplItemVerbatim = match syn::parse2(tokens.clone()) {
            Ok(impl_item) => impl_item,
            Err(_) => unimplemented!("ImplItem::Verbatim `{}`", tokens),
        };

        match impl_item {
            ImplItemVerbatim::Empty => {
                self.hardbreak();
            }
            ImplItemVerbatim::Ellipsis => {
                self.scan_string("...");
                self.hardbreak();
            }
            ImplItemVerbatim::ConstFlexible(impl_item) => {
                self.flexible_item_const(&impl_item);
            }
            ImplItemVerbatim::FnFlexible(impl_item) => {
                self.flexible_item_fn(&impl_item);
            }
            ImplItemVerbatim::TypeFlexible(impl_item) => {
                self.flexible_item_type(&impl_item);
            }
        }
    }

    fn signature(&mut self, signature: &Signature) {
        if signature.constness.is_some() {
            self.scan_string("const ");
        }
        if signature.asyncness.is_some() {
            self.scan_string("async ");
        }
        if signature.unsafety.is_some() {
            self.scan_string("unsafe ");
        }
        if let Some(abi) = &signature.abi {
            self.abi(abi);
        }
        self.scan_string("fn ");
        self.ident(&signature.ident);
        self.generics(&signature.generics);
        self.scan_string("(");
        self.neverbreak();
        self.scan_begin_consistent(0);
        self.zerobreak();
        for input in signature.inputs.iter().delimited() {
            self.fn_arg(&input);
            let is_last = input.is_last && signature.variadic.is_none();
            self.trailing_comma(is_last);
        }
        if let Some(variadic) = &signature.variadic {
            self.variadic(variadic);
            self.zerobreak();
        }
        self.offset(-self.config.indent);
        self.scan_end();
        self.scan_string(")");
        self.scan_begin_consistent(-self.config.indent);
        self.return_type(&signature.output);
        self.scan_end();
    }

    fn fn_arg(&mut self, fn_arg: &FnArg) {
        match fn_arg {
            FnArg::Receiver(receiver) => self.receiver(receiver),
            FnArg::Typed(pat_type) => self.pat_type(pat_type),
        }
    }

    fn receiver(&mut self, receiver: &Receiver) {
        self.outer_attrs(&receiver.attrs);
        if let Some((_ampersand, lifetime)) = &receiver.reference {
            self.scan_string("&");
            if let Some(lifetime) = lifetime {
                self.lifetime(lifetime);
                self.nbsp();
            }
        }
        if receiver.mutability.is_some() {
            self.scan_string("mut ");
        }
        self.scan_string("self");
        if receiver.colon_token.is_some() {
            self.scan_string(": ");
            self.ty(&receiver.ty);
        } else {
            let consistent = match (&receiver.reference, &receiver.mutability, &*receiver.ty) {
                (Some(_), mutability, Type::Reference(ty)) => {
                    mutability.is_some() == ty.mutability.is_some()
                        && match &*ty.elem {
                            Type::Path(ty) => ty.qself.is_none() && ty.path.is_ident("Self"),
                            _ => false,
                        }
                }
                (None, _, Type::Path(ty)) => ty.qself.is_none() && ty.path.is_ident("Self"),
                _ => false,
            };
            if !consistent {
                self.scan_string(": ");
                self.ty(&receiver.ty);
            }
        }
    }

    fn variadic(&mut self, variadic: &Variadic) {
        self.outer_attrs(&variadic.attrs);
        if let Some((pat, _colon)) = &variadic.pat {
            self.pat(pat);
            self.scan_string(": ");
        }
        self.scan_string("...");
    }

    fn static_mutability(&mut self, mutability: &StaticMutability) {
        match mutability {
            #![cfg_attr(all(test, exhaustive), deny(non_exhaustive_omitted_patterns))]
            StaticMutability::Mut(_) => self.scan_string("mut "),
            StaticMutability::None => {}
            _ => unimplemented!("unknown StaticMutability"),
        }
    }
}

#[cfg(feature = "verbatim")]
mod verbatim {
    use crate::engine::Engine;
    use crate::iter::IterDelimited;
    use syn::ext::IdentExt;
    use syn::parse::{ParseStream, Result};
    use syn::{
        braced, token, Attribute, Block, Expr, Generics, Ident, Signature, StaticMutability, Stmt,
        Token, Type, TypeParamBound, Visibility, WhereClause,
    };

    pub struct FlexibleItemConst {
        pub attrs: Vec<Attribute>,
        pub vis: Visibility,
        pub defaultness: bool,
        pub ident: Ident,
        pub generics: Generics,
        pub ty: Type,
        pub value: Option<Expr>,
    }

    pub struct FlexibleItemFn {
        pub attrs: Vec<Attribute>,
        pub vis: Visibility,
        pub defaultness: bool,
        pub sig: Signature,
        pub body: Option<Vec<Stmt>>,
    }

    pub struct FlexibleItemStatic {
        pub attrs: Vec<Attribute>,
        pub vis: Visibility,
        pub mutability: StaticMutability,
        pub ident: Ident,
        pub ty: Option<Type>,
        pub expr: Option<Expr>,
    }

    pub struct FlexibleItemType {
        pub attrs: Vec<Attribute>,
        pub vis: Visibility,
        pub defaultness: bool,
        pub ident: Ident,
        pub generics: Generics,
        pub bounds: Vec<TypeParamBound>,
        pub definition: Option<Type>,
        pub where_clause_after_eq: Option<WhereClause>,
    }

    pub enum WhereClauseLocation {
        // type Ty<T> where T: 'static = T;
        BeforeEq,
        // type Ty<T> = T where T: 'static;
        AfterEq,
        // TODO: goes away once the migration period on rust-lang/rust#89122 is over
        Both,
    }

    impl FlexibleItemConst {
        pub fn parse(
            attrs: Vec<Attribute>,
            vis: Visibility,
            defaultness: bool,
            input: ParseStream,
        ) -> Result<Self> {
            input.parse::<Token![const]>()?;
            let ident = input.call(Ident::parse_any)?;
            let mut generics: Generics = input.parse()?;
            input.parse::<Token![:]>()?;
            let ty: Type = input.parse()?;
            let value = if input.parse::<Option<Token![=]>>()?.is_some() {
                let expr: Expr = input.parse()?;
                Some(expr)
            } else {
                None
            };
            generics.where_clause = input.parse()?;
            input.parse::<Token![;]>()?;

            Ok(FlexibleItemConst {
                attrs,
                vis,
                defaultness,
                ident,
                generics,
                ty,
                value,
            })
        }
    }

    impl FlexibleItemFn {
        pub fn parse(
            mut attrs: Vec<Attribute>,
            vis: Visibility,
            defaultness: bool,
            input: ParseStream,
        ) -> Result<Self> {
            let sig: Signature = input.parse()?;

            let lookahead = input.lookahead1();
            let body = if lookahead.peek(Token![;]) {
                input.parse::<Token![;]>()?;
                None
            } else if lookahead.peek(token::Brace) {
                let content;
                braced!(content in input);
                attrs.extend(content.call(Attribute::parse_inner)?);
                Some(content.call(Block::parse_within)?)
            } else {
                return Err(lookahead.error());
            };

            Ok(FlexibleItemFn {
                attrs,
                vis,
                defaultness,
                sig,
                body,
            })
        }
    }

    impl FlexibleItemStatic {
        pub fn parse(attrs: Vec<Attribute>, vis: Visibility, input: ParseStream) -> Result<Self> {
            input.parse::<Token![static]>()?;
            let mutability: StaticMutability = input.parse()?;
            let ident = input.parse()?;

            let lookahead = input.lookahead1();
            let has_type = lookahead.peek(Token![:]);
            let has_expr = lookahead.peek(Token![=]);
            if !has_type && !has_expr {
                return Err(lookahead.error());
            }

            let ty: Option<Type> = if has_type {
                input.parse::<Token![:]>()?;
                input.parse().map(Some)?
            } else {
                None
            };

            let expr: Option<Expr> = if input.parse::<Option<Token![=]>>()?.is_some() {
                input.parse().map(Some)?
            } else {
                None
            };

            input.parse::<Token![;]>()?;

            Ok(FlexibleItemStatic {
                attrs,
                vis,
                mutability,
                ident,
                ty,
                expr,
            })
        }
    }

    impl FlexibleItemType {
        pub fn parse(
            attrs: Vec<Attribute>,
            vis: Visibility,
            defaultness: bool,
            input: ParseStream,
            where_clause_location: WhereClauseLocation,
        ) -> Result<Self> {
            input.parse::<Token![type]>()?;
            let ident: Ident = input.parse()?;
            let mut generics: Generics = input.parse()?;

            let mut bounds = Vec::new();
            if input.parse::<Option<Token![:]>>()?.is_some() {
                loop {
                    if input.peek(Token![where]) || input.peek(Token![=]) || input.peek(Token![;]) {
                        break;
                    }
                    bounds.push(input.parse::<TypeParamBound>()?);
                    if input.peek(Token![where]) || input.peek(Token![=]) || input.peek(Token![;]) {
                        break;
                    }
                    input.parse::<Token![+]>()?;
                }
            }

            match where_clause_location {
                WhereClauseLocation::BeforeEq | WhereClauseLocation::Both => {
                    generics.where_clause = input.parse()?;
                }
                WhereClauseLocation::AfterEq => {}
            }

            let definition = if input.parse::<Option<Token![=]>>()?.is_some() {
                Some(input.parse()?)
            } else {
                None
            };

            let where_clause_after_eq = match where_clause_location {
                WhereClauseLocation::AfterEq | WhereClauseLocation::Both
                    if generics.where_clause.is_none() =>
                {
                    input.parse()?
                }
                _ => None,
            };

            input.parse::<Token![;]>()?;

            Ok(FlexibleItemType {
                attrs,
                vis,
                defaultness,
                ident,
                generics,
                bounds,
                definition,
                where_clause_after_eq,
            })
        }
    }

    impl Engine {
        pub fn flexible_item_const(&mut self, item: &FlexibleItemConst) {
            self.outer_attrs(&item.attrs);
            self.scan_begin_consistent(self.config.indent);
            self.visibility(&item.vis);
            if item.defaultness {
                self.scan_string("default ");
            }
            self.scan_string("const ");
            self.ident(&item.ident);
            self.generics(&item.generics);
            self.scan_string(": ");
            self.scan_begin_consistent(-self.config.indent);
            self.ty(&item.ty);
            self.scan_end();
            if let Some(value) = &item.value {
                self.scan_string(" = ");
                self.neverbreak();
                self.scan_begin_inconsistent(-self.config.indent);
                self.expr(value);
                self.scan_end();
            }
            self.where_clause_oneline_semi(&item.generics.where_clause);
            self.scan_end();
            self.hardbreak();
        }

        pub fn flexible_item_fn(&mut self, item: &FlexibleItemFn) {
            self.outer_attrs(&item.attrs);
            self.scan_begin_consistent(self.config.indent);
            self.visibility(&item.vis);
            if item.defaultness {
                self.scan_string("default ");
            }
            self.signature(&item.sig);
            if let Some(body) = &item.body {
                self.where_clause_for_body(&item.sig.generics.where_clause);
                self.scan_string("{");
                self.hardbreak_if_nonempty();
                self.inner_attrs(&item.attrs);
                for stmt in body {
                    self.stmt(stmt);
                }
                self.offset(-self.config.indent);
                self.scan_end();
                self.scan_string("}");
            } else {
                self.where_clause_semi(&item.sig.generics.where_clause);
                self.scan_end();
            }
            self.hardbreak();
        }

        pub fn flexible_item_static(&mut self, item: &FlexibleItemStatic) {
            self.outer_attrs(&item.attrs);
            self.scan_begin_consistent(0);
            self.visibility(&item.vis);
            self.scan_string("static ");
            self.static_mutability(&item.mutability);
            self.ident(&item.ident);
            if let Some(ty) = &item.ty {
                self.scan_string(": ");
                self.ty(ty);
            }
            if let Some(expr) = &item.expr {
                self.scan_string(" = ");
                self.neverbreak();
                self.expr(expr);
            }
            self.scan_string(";");
            self.scan_end();
            self.hardbreak();
        }

        pub fn flexible_item_type(&mut self, item: &FlexibleItemType) {
            self.outer_attrs(&item.attrs);
            self.scan_begin_consistent(self.config.indent);
            self.visibility(&item.vis);
            if item.defaultness {
                self.scan_string("default ");
            }
            self.scan_string("type ");
            self.ident(&item.ident);
            self.generics(&item.generics);
            for bound in item.bounds.iter().delimited() {
                if bound.is_first {
                    self.scan_string(": ");
                } else {
                    self.space();
                    self.scan_string("+ ");
                }
                self.type_param_bound(&bound);
            }
            if let Some(definition) = &item.definition {
                self.where_clause_oneline(&item.generics.where_clause);
                self.scan_string("= ");
                self.neverbreak();
                self.scan_begin_inconsistent(-self.config.indent);
                self.ty(definition);
                self.scan_end();
                self.where_clause_oneline_semi(&item.where_clause_after_eq);
            } else {
                self.where_clause_oneline_semi(&item.generics.where_clause);
            }
            self.scan_end();
            self.hardbreak();
        }
    }
}
