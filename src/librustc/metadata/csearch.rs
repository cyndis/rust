// Copyright 2012 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// Searching for information from the cstore

use core::prelude::*;

use metadata::common::*;
use metadata::cstore;
use metadata::decoder;
use metadata;
use middle::ty;

use core::dvec::DVec;
use core::vec;
use reader = std::ebml::reader;
use std::ebml;
use std::oldmap::HashMap;
use syntax::ast;
use syntax::ast_map;
use syntax::codemap::dummy_sp;
use syntax::ast_util;
use syntax::diagnostic::expect;
use syntax::diagnostic::span_handler;

pub struct ProvidedTraitMethodInfo {
    ty: ty::method,
    def_id: ast::def_id
}

pub struct StaticMethodInfo {
    ident: ast::ident,
    def_id: ast::def_id,
    purity: ast::purity
}

pub fn get_symbol(cstore: @mut cstore::CStore, def: ast::def_id) -> ~str {
    let cdata = cstore::get_crate_data(cstore, def.crate).data;
    return decoder::get_symbol(cdata, def.node);
}

pub fn get_type_param_count(cstore: @mut cstore::CStore, def: ast::def_id)
                         -> uint {
    let cdata = cstore::get_crate_data(cstore, def.crate).data;
    return decoder::get_type_param_count(cdata, def.node);
}

/// Iterates over all the language items in the given crate.
pub fn each_lang_item(cstore: @mut cstore::CStore,
                      cnum: ast::crate_num,
                      f: &fn(ast::node_id, uint) -> bool) {
    let crate_data = cstore::get_crate_data(cstore, cnum);
    decoder::each_lang_item(crate_data, f)
}

/// Iterates over all the paths in the given crate.
pub fn each_path(cstore: @mut cstore::CStore,
                 cnum: ast::crate_num,
                 f: &fn(&str, decoder::def_like) -> bool) {
    let crate_data = cstore::get_crate_data(cstore, cnum);
    let get_crate_data: decoder::GetCrateDataCb = |cnum| {
        cstore::get_crate_data(cstore, cnum)
    };
    decoder::each_path(cstore.intr, crate_data, get_crate_data, f);
}

pub fn get_item_path(tcx: ty::ctxt, def: ast::def_id) -> ast_map::path {
    let cstore = tcx.cstore;
    let cdata = cstore::get_crate_data(cstore, def.crate);
    let path = decoder::get_item_path(cstore.intr, cdata, def.node);

    // FIXME #1920: This path is not always correct if the crate is not linked
    // into the root namespace.
    vec::append(~[ast_map::path_mod(tcx.sess.ident_of(
        /*bad*/copy cdata.name))], path)
}

pub enum found_ast {
    found(ast::inlined_item),
    found_parent(ast::def_id, ast::inlined_item),
    not_found,
}

// Finds the AST for this item in the crate metadata, if any.  If the item was
// not marked for inlining, then the AST will not be present and hence none
// will be returned.
pub fn maybe_get_item_ast(tcx: ty::ctxt, def: ast::def_id,
                          decode_inlined_item: decoder::decode_inlined_item)
                       -> found_ast {
    let cstore = tcx.cstore;
    let cdata = cstore::get_crate_data(cstore, def.crate);
    decoder::maybe_get_item_ast(cstore.intr, cdata, tcx, def.node,
                                decode_inlined_item)
}

pub fn get_enum_variants(tcx: ty::ctxt, def: ast::def_id)
                      -> ~[ty::VariantInfo] {
    let cstore = tcx.cstore;
    let cdata = cstore::get_crate_data(cstore, def.crate);
    return decoder::get_enum_variants(cstore.intr, cdata, def.node, tcx)
}

pub fn get_impls_for_mod(cstore: @mut cstore::CStore, def: ast::def_id,
                         name: Option<ast::ident>)
                      -> @~[@decoder::_impl] {
    let cdata = cstore::get_crate_data(cstore, def.crate);
    do decoder::get_impls_for_mod(cstore.intr, cdata, def.node, name) |cnum| {
        cstore::get_crate_data(cstore, cnum)
    }
}

pub fn get_trait_methods(tcx: ty::ctxt,
                         def: ast::def_id)
                      -> @~[ty::method] {
    let cstore = tcx.cstore;
    let cdata = cstore::get_crate_data(cstore, def.crate);
    decoder::get_trait_methods(cstore.intr, cdata, def.node, tcx)
}

pub fn get_provided_trait_methods(tcx: ty::ctxt,
                                  def: ast::def_id)
                               -> ~[ProvidedTraitMethodInfo] {
    let cstore = tcx.cstore;
    let cdata = cstore::get_crate_data(cstore, def.crate);
    decoder::get_provided_trait_methods(cstore.intr, cdata, def.node, tcx)
}

pub fn get_supertraits(tcx: ty::ctxt, def: ast::def_id) -> ~[ty::t] {
    let cstore = tcx.cstore;
    let cdata = cstore::get_crate_data(cstore, def.crate);
    decoder::get_supertraits(cdata, def.node, tcx)
}

pub fn get_method_names_if_trait(cstore: @mut cstore::CStore,
                                 def: ast::def_id)
                              -> Option<@DVec<(ast::ident, ast::self_ty_)>> {
    let cdata = cstore::get_crate_data(cstore, def.crate);
    return decoder::get_method_names_if_trait(cstore.intr, cdata, def.node);
}

pub fn get_type_name_if_impl(cstore: @mut cstore::CStore, def: ast::def_id)
                          -> Option<ast::ident> {
    let cdata = cstore::get_crate_data(cstore, def.crate);
    decoder::get_type_name_if_impl(cstore.intr, cdata, def.node)
}

pub fn get_static_methods_if_impl(cstore: @mut cstore::CStore,
                                  def: ast::def_id)
                               -> Option<~[StaticMethodInfo]> {
    let cdata = cstore::get_crate_data(cstore, def.crate);
    decoder::get_static_methods_if_impl(cstore.intr, cdata, def.node)
}

pub fn get_item_attrs(cstore: @mut cstore::CStore,
                      def_id: ast::def_id,
                      f: fn(~[@ast::meta_item])) {
    let cdata = cstore::get_crate_data(cstore, def_id.crate);
    decoder::get_item_attrs(cdata, def_id.node, f)
}

pub fn get_struct_fields(tcx: ty::ctxt, def: ast::def_id) -> ~[ty::field_ty] {
    let cstore = tcx.cstore;
    let cdata = cstore::get_crate_data(cstore, def.crate);
    decoder::get_struct_fields(cstore.intr, cdata, def.node)
}

pub fn get_type(tcx: ty::ctxt,
                def: ast::def_id)
             -> ty::ty_param_bounds_and_ty {
    let cstore = tcx.cstore;
    let cdata = cstore::get_crate_data(cstore, def.crate);
    decoder::get_type(cdata, def.node, tcx)
}

pub fn get_region_param(cstore: @mut metadata::cstore::CStore,
                        def: ast::def_id) -> Option<ty::region_variance> {
    let cdata = cstore::get_crate_data(cstore, def.crate);
    return decoder::get_region_param(cdata, def.node);
}

pub fn get_field_type(tcx: ty::ctxt, class_id: ast::def_id,
                      def: ast::def_id) -> ty::ty_param_bounds_and_ty {
    let cstore = tcx.cstore;
    let cdata = cstore::get_crate_data(cstore, class_id.crate);
    let all_items = reader::get_doc(reader::Doc(cdata.data), tag_items);
    debug!("Looking up %?", class_id);
    let class_doc = expect(tcx.diag,
                           decoder::maybe_find_item(class_id.node, all_items),
                           || fmt!("get_field_type: class ID %? not found",
                                   class_id) );
    debug!("looking up %? : %?", def, class_doc);
    let the_field = expect(tcx.diag,
        decoder::maybe_find_item(def.node, class_doc),
        || fmt!("get_field_type: in class %?, field ID %? not found",
                 class_id, def) );
    debug!("got field data %?", the_field);
    let ty = decoder::item_type(def, the_field, tcx, cdata);
    return {bounds: @~[],
            region_param: None,
            ty: ty};
}

// Given a def_id for an impl or class, return the traits it implements,
// or the empty vector if it's not for an impl or for a class that implements
// traits
pub fn get_impl_traits(tcx: ty::ctxt, def: ast::def_id) -> ~[ty::t] {
    let cstore = tcx.cstore;
    let cdata = cstore::get_crate_data(cstore, def.crate);
    decoder::get_impl_traits(cdata, def.node, tcx)
}

pub fn get_impl_method(cstore: @mut cstore::CStore,
                       def: ast::def_id,
                       mname: ast::ident)
                    -> ast::def_id {
    let cdata = cstore::get_crate_data(cstore, def.crate);
    decoder::get_impl_method(cstore.intr, cdata, def.node, mname)
}

/* If def names a class with a dtor, return it. Otherwise, return none. */
pub fn struct_dtor(cstore: @mut cstore::CStore, def: ast::def_id)
    -> Option<ast::def_id> {
    let cdata = cstore::get_crate_data(cstore, def.crate);
    decoder::struct_dtor(cdata, def.node)
}
// Local Variables:
// mode: rust
// fill-column: 78;
// indent-tabs-mode: nil
// c-basic-offset: 4
// buffer-file-coding-system: utf-8-unix
// End:
