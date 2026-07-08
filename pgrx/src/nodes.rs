//LICENSE Portions Copyright 2019-2021 ZomboDB, LLC.
//LICENSE
//LICENSE Portions Copyright 2021-2023 Technology Concepts & Design, Inc.
//LICENSE
//LICENSE Portions Copyright 2023-2023 PgCentral Foundation, Inc. <contact@pgcentral.org>
//LICENSE
//LICENSE All rights reserved.
//LICENSE
//LICENSE Use of this source code is governed by the MIT license that can be found in the LICENSE file.
//! Helper functions and such for Postgres' various query tree `Node`s

use crate as pgrx;
use crate::{pg_guard, pg_sys, void_mut_ptr};
use core::ptr::NonNull;

/// #define IsA(nodeptr,_type_)            (nodeTag(nodeptr) == T_##_type_)
#[inline]
pub unsafe fn is_a(nodeptr: *mut pg_sys::Node, tag: pg_sys::NodeTag) -> bool {
    !nodeptr.is_null() && nodeptr.as_ref().unwrap().type_ == tag
}

/// Convert a [pg_sys::Node] into its textual representation
///
/// ### Safety
///
/// We cannot guarantee the provided `nodeptr` is a valid pointer
pub unsafe fn node_to_string<'a>(nodeptr: *mut pg_sys::Node) -> Option<&'a str> {
    if nodeptr.is_null() {
        None
    } else {
        let string = pg_sys::nodeToString(nodeptr as crate::void_ptr);
        if string.is_null() {
            None
        } else {
            Some(
                core::ffi::CStr::from_ptr(string)
                    .to_str()
                    .expect("unable to convert Node into a &str"),
            )
        }
    }
}

pub enum Walk {
    Continue,
    Stop,
}

pub trait PlanStateVisitor {
    fn visit(&mut self, planstate: &pg_sys::PlanState) -> Walk;
}

#[doc(alias = "planstate_tree_walker")]
pub fn walk_planstate_tree<V>(planstate: &pg_sys::PlanState, visitor: &mut V) -> Walk
where
    V: PlanStateVisitor,
{
    // The walker should return "false" to continue or "true" to abort the walk.
    #[doc(alias = "planstate_tree_walker_callback")]
    type Walker = unsafe extern "C-unwind" fn(*mut pg_sys::PlanState, void_mut_ptr) -> bool;

    #[pg_guard]
    unsafe extern "C-unwind" fn callback<V: PlanStateVisitor>(
        planstate: *mut pg_sys::PlanState,
        context: void_mut_ptr,
    ) -> bool {
        // Proceed past null pointers; `as_ref` returns `None` for null.
        let Some(planstate) = (unsafe { planstate.as_ref() }) else { return false };
        let visitor = unsafe { (context as *mut V).as_mut_unchecked() };

        matches!(visitor.visit(planstate), Walk::Stop)
    }

    let context = NonNull::from_mut(visitor).as_ptr() as void_mut_ptr;
    let planstate = NonNull::from_ref(planstate).as_ptr();
    let walker = Some(callback::<V> as Walker);
    let done = unsafe { pg_sys::planstate_tree_walker(planstate, walker, context) };

    if !done { Walk::Continue } else { Walk::Stop }
}

pub trait TreeVisitor {
    fn visit(&mut self, node: &pg_sys::Node) -> Walk;
}

// The walker should return "false" to continue or "true" to abort the walk.
#[doc(alias = "tree_walker_callback")]
type TreeWalker = unsafe extern "C-unwind" fn(*mut pg_sys::Node, void_mut_ptr) -> bool;

#[doc(alias = "expression_tree_walker")]
pub fn walk_expression_tree<N: pg_sys::PgNode, V>(node: &N, visitor: &mut V) -> Walk
where
    V: TreeVisitor,
{
    #[pg_guard]
    unsafe extern "C-unwind" fn callback<V: TreeVisitor>(
        node: *mut pg_sys::Node,
        context: void_mut_ptr,
    ) -> bool {
        // Proceed past null pointers; `as_ref` returns `None` for null.
        let Some(node) = (unsafe { node.as_ref() }) else { return false };
        let visitor = unsafe { (context as *mut V).as_mut_unchecked() };

        matches!(visitor.visit(node), Walk::Stop)
    }

    let context = NonNull::from_mut(visitor).as_ptr() as void_mut_ptr;
    let node = NonNull::from_ref(node.as_node()).as_ptr();
    let walker = Some(callback::<V> as TreeWalker);
    let done = unsafe { pg_sys::expression_tree_walker(node, walker, context) };

    if !done { Walk::Continue } else { Walk::Stop }
}

bitflags! {
    #[derive(Default, Copy, Clone)]
    /// Flags to control behaviour during [walk_query_tree].
    pub struct WalkQueryTree: i32 {
        /// Do not copy top Query
        const DONT_COPY_QUERY = pg_sys::QTW_DONT_COPY_QUERY as i32;
        /// Examine RTE nodes after their content
        const EXAMINE_RTES_AFTER = pg_sys::QTW_EXAMINE_RTES_AFTER as i32;
        /// Examine RTE nodes before their content
        const EXAMINE_RTES_BEFORE = pg_sys::QTW_EXAMINE_RTES_BEFORE as i32;
        /// Include SortGroupClause lists
        const EXAMINE_SORT_GROUP = pg_sys::QTW_EXAMINE_SORTGROUP as i32;
        /// Ignore subqueries in CTE list
        const IGNORE_CTE_SUBQUERIES = pg_sys::QTW_IGNORE_CTE_SUBQUERIES as i32;
        /// Ignore subqueries in rtable
        const IGNORE_RTABLE_SUBQUERIES = pg_sys::QTW_IGNORE_RT_SUBQUERIES as i32;
        /// Ignore GROUP expressions list
        #[cfg(feature = "pg18")]
        const IGNORE_GROUP_EXPRS = pg_sys::QTW_IGNORE_GROUPEXPRS as i32;
        /// Ignore JOIN alias var lists
        const IGNORE_JOIN_ALIASES = pg_sys::QTW_IGNORE_JOINALIASES as i32;
        /// Skip rangetable entirely
        const IGNORE_RANGE_TABLE = pg_sys::QTW_IGNORE_RANGE_TABLE as i32;
    }
}

#[doc(alias = "query_tree_walker")]
pub fn walk_query_tree<V>(query: &pg_sys::Query, visitor: &mut V, options: WalkQueryTree) -> Walk
where
    V: TreeVisitor,
{
    #[pg_guard]
    unsafe extern "C-unwind" fn callback<V: TreeVisitor>(
        node: *mut pg_sys::Node,
        context: void_mut_ptr,
    ) -> bool {
        // Proceed past null pointers; `as_ref` returns `None` for null.
        let Some(node) = (unsafe { node.as_ref() }) else { return false };
        let visitor = unsafe { (context as *mut V).as_mut_unchecked() };

        matches!(visitor.visit(node), Walk::Stop)
    }

    let context = NonNull::from_mut(visitor).as_ptr() as void_mut_ptr;
    let query = NonNull::from_ref(query).as_ptr();
    let walker = Some(callback::<V> as TreeWalker);
    let done = unsafe { pg_sys::query_tree_walker(query, walker, context, options.bits()) };

    if !done { Walk::Continue } else { Walk::Stop }
}

#[doc(alias = "raw_expression_tree_walker")]
pub fn walk_raw_expression_tree<N: pg_sys::PgNode, V>(node: &N, visitor: &mut V) -> Walk
where
    V: TreeVisitor,
{
    #[pg_guard]
    unsafe extern "C-unwind" fn callback<V: TreeVisitor>(
        node: *mut pg_sys::Node,
        context: void_mut_ptr,
    ) -> bool {
        // Proceed past null pointers; `as_ref` returns `None` for null.
        let Some(node) = (unsafe { node.as_ref() }) else { return false };
        let visitor = unsafe { (context as *mut V).as_mut_unchecked() };

        matches!(visitor.visit(node), Walk::Stop)
    }

    let context = NonNull::from_mut(visitor).as_ptr() as void_mut_ptr;
    let node = NonNull::from_ref(node.as_node()).as_ptr();
    let walker = Some(callback::<V> as TreeWalker);
    let done = unsafe { pg_sys::raw_expression_tree_walker(node, walker, context) };

    if !done { Walk::Continue } else { Walk::Stop }
}
