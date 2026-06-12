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

use crate::pg_sys;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraversalControl {
    Continue,
    Break,
}

impl TraversalControl {
    #[inline]
    pub fn is_break(&self) -> bool {
        matches!(self, TraversalControl::Break)
    }
}

pub trait AsPgNode {
    fn as_node(&self) -> &pg_sys::Node;
    fn as_node_mut(&mut self) -> &mut pg_sys::Node;
}

pub trait HasNodeTag: AsPgNode {
    const NODE_TAGS: &'static [pg_sys::NodeTag];
}

macro_rules! impl_has_node_tag {
    ($struct_name:ident, [$($tag_variant:ident),+]) => {
        impl AsPgNode for pg_sys::$struct_name {
            #[inline]
            fn as_node(&self) -> &pg_sys::Node {
                unsafe { &*(self as *const pg_sys::$struct_name as *const pg_sys::Node) }
            }
            #[inline]
            fn as_node_mut(&mut self) -> &mut pg_sys::Node {
                unsafe { &mut *(self as *mut pg_sys::$struct_name as *mut pg_sys::Node) }
            }
        }
        impl HasNodeTag for pg_sys::$struct_name {
            const NODE_TAGS: &'static [pg_sys::NodeTag] = &[
                $(pg_sys::NodeTag::$tag_variant),+
            ];
        }
    };
    ($struct_name:ident, $tag_variant:ident) => {
        impl AsPgNode for pg_sys::$struct_name {
            #[inline]
            fn as_node(&self) -> &pg_sys::Node {
                unsafe { &*(self as *const pg_sys::$struct_name as *const pg_sys::Node) }
            }
            #[inline]
            fn as_node_mut(&mut self) -> &mut pg_sys::Node {
                unsafe { &mut *(self as *mut pg_sys::$struct_name as *mut pg_sys::Node) }
            }
        }
        impl HasNodeTag for pg_sys::$struct_name {
            const NODE_TAGS: &'static [pg_sys::NodeTag] = &[pg_sys::NodeTag::$tag_variant];
        }
    };
}

impl_has_node_tag!(CreateStmt, T_CreateStmt);
impl_has_node_tag!(AlterTableStmt, T_AlterTableStmt);
impl_has_node_tag!(AlterTableCmd, T_AlterTableCmd);
impl_has_node_tag!(DropStmt, T_DropStmt);
impl_has_node_tag!(RangeVar, T_RangeVar);
impl_has_node_tag!(List, [T_List, T_IntList, T_OidList, T_XidList]);
impl_has_node_tag!(TruncateStmt, T_TruncateStmt);
impl_has_node_tag!(IndexStmt, T_IndexStmt);
impl_has_node_tag!(RangeTblEntry, T_RangeTblEntry);
impl_has_node_tag!(SeqScan, T_SeqScan);
impl_has_node_tag!(ModifyTable, T_ModifyTable);
impl_has_node_tag!(Agg, T_Agg);
impl_has_node_tag!(Sort, T_Sort);
impl_has_node_tag!(TargetEntry, T_TargetEntry);
impl_has_node_tag!(Append, T_Append);
impl_has_node_tag!(MergeAppend, T_MergeAppend);
impl_has_node_tag!(Query, T_Query);

impl AsPgNode for pg_sys::Plan {
    #[inline]
    fn as_node(&self) -> &pg_sys::Node {
        unsafe { &*(self as *const pg_sys::Plan as *const pg_sys::Node) }
    }
    #[inline]
    fn as_node_mut(&mut self) -> &mut pg_sys::Node {
        unsafe { &mut *(self as *mut pg_sys::Plan as *mut pg_sys::Node) }
    }
}

impl AsPgNode for pg_sys::Scan {
    #[inline]
    fn as_node(&self) -> &pg_sys::Node {
        unsafe { &*(self as *const pg_sys::Scan as *const pg_sys::Node) }
    }
    #[inline]
    fn as_node_mut(&mut self) -> &mut pg_sys::Node {
        unsafe { &mut *(self as *mut pg_sys::Scan as *mut pg_sys::Node) }
    }
}

impl AsPgNode for pg_sys::Expr {
    #[inline]
    fn as_node(&self) -> &pg_sys::Node {
        unsafe { &*(self as *const pg_sys::Expr as *const pg_sys::Node) }
    }
    #[inline]
    fn as_node_mut(&mut self) -> &mut pg_sys::Node {
        unsafe { &mut *(self as *mut pg_sys::Expr as *mut pg_sys::Node) }
    }
}

pub trait PgNodeTryCast<T> {
    fn try_cast_from(node: T) -> Option<Self>
    where
        Self: Sized;
}

impl<'a, B: HasNodeTag> PgNodeTryCast<&'a pg_sys::Node> for &'a B {
    fn try_cast_from(node: &'a pg_sys::Node) -> Option<Self> {
        if B::NODE_TAGS.contains(&node.type_) {
            unsafe { Some(&*(node as *const pg_sys::Node as *const B)) }
        } else {
            None
        }
    }
}

impl<'a, B: HasNodeTag> PgNodeTryCast<&'a mut pg_sys::Node> for &'a mut B {
    fn try_cast_from(node: &'a mut pg_sys::Node) -> Option<Self> {
        if B::NODE_TAGS.contains(&node.type_) {
            unsafe { Some(&mut *(node as *mut pg_sys::Node as *mut B)) }
        } else {
            None
        }
    }
}

pub trait PgNodeWalk {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> TraversalControl;
}

pub trait PlannedStmtVisitor {
    // Generic catch-all — defaults to Continue (no-op)
    fn visit_node(&mut self, _node: &pg_sys::Node) -> TraversalControl {
        TraversalControl::Continue
    }

    // Generic list catch-all — used only from Node::walk for unrecognised lists
    fn visit_list(&mut self, list: &pg_sys::List) -> TraversalControl {
        if self.visit_node(list.as_node()).is_break() {
            return TraversalControl::Break;
        }
        list.walk(self)
    }

    // Statement-level hooks
    fn visit_create_stmt(&mut self, stmt: &pg_sys::CreateStmt) -> TraversalControl {
        if self.visit_node(stmt.as_node()).is_break() { return TraversalControl::Break; }
        stmt.walk(self)
    }

    fn visit_alter_table_stmt(&mut self, stmt: &pg_sys::AlterTableStmt) -> TraversalControl {
        if self.visit_node(stmt.as_node()).is_break() { return TraversalControl::Break; }
        stmt.walk(self)
    }

    fn visit_alter_table_cmd(&mut self, cmd: &pg_sys::AlterTableCmd) -> TraversalControl {
        if self.visit_node(cmd.as_node()).is_break() { return TraversalControl::Break; }
        cmd.walk(self)
    }

    fn visit_drop_stmt(&mut self, stmt: &pg_sys::DropStmt) -> TraversalControl {
        if self.visit_node(stmt.as_node()).is_break() { return TraversalControl::Break; }
        stmt.walk(self)
    }

    fn visit_truncate_stmt(&mut self, stmt: &pg_sys::TruncateStmt) -> TraversalControl {
        if self.visit_node(stmt.as_node()).is_break() { return TraversalControl::Break; }
        stmt.walk(self)
    }

    fn visit_index_stmt(&mut self, stmt: &pg_sys::IndexStmt) -> TraversalControl {
        if self.visit_node(stmt.as_node()).is_break() { return TraversalControl::Break; }
        stmt.walk(self)
    }

    // Shared leaf hook
    fn visit_range_var(&mut self, range_var: &pg_sys::RangeVar) -> TraversalControl {
        if self.visit_node(range_var.as_node()).is_break() { return TraversalControl::Break; }
        range_var.walk(self)
    }

    // CreateStmt field hooks
    fn visit_create_table_elts(&mut self, list: &pg_sys::List) -> TraversalControl {
        self.visit_list(list)
    }
    fn visit_create_inh_relations(&mut self, list: &pg_sys::List) -> TraversalControl {
        self.visit_list(list)
    }
    fn visit_create_constraints(&mut self, list: &pg_sys::List) -> TraversalControl {
        self.visit_list(list)
    }
    fn visit_create_options(&mut self, list: &pg_sys::List) -> TraversalControl {
        self.visit_list(list)
    }

    // AlterTableStmt field hook
    fn visit_alter_table_cmds(&mut self, list: &pg_sys::List) -> TraversalControl {
        self.visit_list(list)
    }

    // DropStmt field hook
    fn visit_drop_objects(&mut self, list: &pg_sys::List) -> TraversalControl {
        self.visit_list(list)
    }

    // TruncateStmt field hook
    fn visit_truncate_relations(&mut self, list: &pg_sys::List) -> TraversalControl {
        self.visit_list(list)
    }

    // IndexStmt field hooks — each is distinct, allowing unambiguous overrides
    fn visit_index_params(&mut self, list: &pg_sys::List) -> TraversalControl {
        self.visit_list(list)
    }
    fn visit_index_including_params(&mut self, list: &pg_sys::List) -> TraversalControl {
        self.visit_list(list)
    }
    fn visit_index_options(&mut self, list: &pg_sys::List) -> TraversalControl {
        self.visit_list(list)
    }

    // DML / Plan hooks
    fn visit_plan(&mut self, plan: &pg_sys::Plan) -> TraversalControl {
        if self.visit_node(plan.as_node()).is_break() { return TraversalControl::Break; }
        plan.walk(self)
    }

    fn visit_scan(&mut self, scan: &pg_sys::Scan) -> TraversalControl {
        if self.visit_plan(&scan.plan).is_break() { return TraversalControl::Break; }
        // Scan has no Node* fields, but contains scanrelid (RTE index).
        TraversalControl::Continue
    }

    fn visit_seq_scan(&mut self, scan: &pg_sys::SeqScan) -> TraversalControl {
        if self.visit_scan(&scan.scan).is_break() { return TraversalControl::Break; }
        scan.walk(self)
    }

    fn visit_modify_table(&mut self, mt: &pg_sys::ModifyTable) -> TraversalControl {
        if self.visit_plan(&mt.plan).is_break() { return TraversalControl::Break; }
        mt.walk(self)
    }

    fn visit_agg(&mut self, agg: &pg_sys::Agg) -> TraversalControl {
        if self.visit_plan(&agg.plan).is_break() { return TraversalControl::Break; }
        agg.walk(self)
    }

    fn visit_sort(&mut self, sort: &pg_sys::Sort) -> TraversalControl {
        if self.visit_plan(&sort.plan).is_break() { return TraversalControl::Break; }
        sort.walk(self)
    }

    fn visit_append(&mut self, append: &pg_sys::Append) -> TraversalControl {
        if self.visit_plan(&append.plan).is_break() { return TraversalControl::Break; }
        append.walk(self)
    }

    fn visit_merge_append(&mut self, ma: &pg_sys::MergeAppend) -> TraversalControl {
        if self.visit_plan(&ma.plan).is_break() { return TraversalControl::Break; }
        ma.walk(self)
    }

    fn visit_range_tbl_entry(&mut self, rte: &pg_sys::RangeTblEntry) -> TraversalControl {
        if self.visit_node(rte.as_node()).is_break() { return TraversalControl::Break; }
        rte.walk(self)
    }

    fn visit_target_entry(&mut self, te: &pg_sys::TargetEntry) -> TraversalControl {
        if self.visit_node(te.as_node()).is_break() { return TraversalControl::Break; }
        te.walk(self)
    }

    fn visit_query(&mut self, query: &pg_sys::Query) -> TraversalControl {
        if self.visit_node(query.as_node()).is_break() { return TraversalControl::Break; }
        query.walk(self)
    }

    // PlannedStmt field hooks
    fn visit_planned_stmt_rtable(&mut self, list: &pg_sys::List) -> TraversalControl {
        self.visit_list(list)
    }
    fn visit_planned_stmt_subplans(&mut self, list: &pg_sys::List) -> TraversalControl {
        self.visit_list(list)
    }
    fn visit_planned_stmt_plan_tree(&mut self, plan: &pg_sys::Plan) -> TraversalControl {
        plan.as_node().walk(self)
    }

    // Plan field hooks
    fn visit_plan_target_list(&mut self, list: &pg_sys::List) -> TraversalControl {
        self.visit_list(list)
    }
    fn visit_plan_qual(&mut self, list: &pg_sys::List) -> TraversalControl {
        self.visit_list(list)
    }

    // ModifyTable field hooks
    fn visit_modify_table_result_relations(&mut self, list: &pg_sys::List) -> TraversalControl {
        self.visit_list(list)
    }

    // Append field hooks
    fn visit_append_plans(&mut self, list: &pg_sys::List) -> TraversalControl {
        self.visit_list(list)
    }

    // MergeAppend field hooks
    fn visit_merge_append_plans(&mut self, list: &pg_sys::List) -> TraversalControl {
        self.visit_list(list)
    }
}

impl PgNodeWalk for pg_sys::Node {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> TraversalControl {
        match self.type_ {
            pg_sys::NodeTag::T_CreateStmt => {
                let stmt = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::CreateStmt) };
                visitor.visit_create_stmt(stmt)
            }
            pg_sys::NodeTag::T_AlterTableStmt => {
                let stmt = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::AlterTableStmt) };
                visitor.visit_alter_table_stmt(stmt)
            }
            pg_sys::NodeTag::T_AlterTableCmd => {
                let cmd = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::AlterTableCmd) };
                visitor.visit_alter_table_cmd(cmd)
            }
            pg_sys::NodeTag::T_DropStmt => {
                let stmt = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::DropStmt) };
                visitor.visit_drop_stmt(stmt)
            }
            pg_sys::NodeTag::T_TruncateStmt => {
                let stmt = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::TruncateStmt) };
                visitor.visit_truncate_stmt(stmt)
            }
            pg_sys::NodeTag::T_IndexStmt => {
                let stmt = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::IndexStmt) };
                visitor.visit_index_stmt(stmt)
            }
            pg_sys::NodeTag::T_RangeVar => {
                let rv = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::RangeVar) };
                visitor.visit_range_var(rv)
            }
            pg_sys::NodeTag::T_List
            | pg_sys::NodeTag::T_IntList
            | pg_sys::NodeTag::T_OidList
            | pg_sys::NodeTag::T_XidList => {
                let list = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::List) };
                visitor.visit_list(list)
            }
            pg_sys::NodeTag::T_RangeTblEntry => {
                let rte = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::RangeTblEntry) };
                visitor.visit_range_tbl_entry(rte)
            }
            pg_sys::NodeTag::T_SeqScan => {
                let scan = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::SeqScan) };
                visitor.visit_seq_scan(scan)
            }
            pg_sys::NodeTag::T_ModifyTable => {
                let mt = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::ModifyTable) };
                visitor.visit_modify_table(mt)
            }
            pg_sys::NodeTag::T_Agg => {
                let agg = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::Agg) };
                visitor.visit_agg(agg)
            }
            pg_sys::NodeTag::T_Sort => {
                let sort = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::Sort) };
                visitor.visit_sort(sort)
            }
            pg_sys::NodeTag::T_TargetEntry => {
                let te = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::TargetEntry) };
                visitor.visit_target_entry(te)
            }
            pg_sys::NodeTag::T_Append => {
                let append = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::Append) };
                visitor.visit_append(append)
            }
            pg_sys::NodeTag::T_MergeAppend => {
                let ma = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::MergeAppend) };
                visitor.visit_merge_append(ma)
            }
            pg_sys::NodeTag::T_Query => {
                let query = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::Query) };
                visitor.visit_query(query)
            }
            _ => visitor.visit_node(self),
        }
    }
}

impl PgNodeWalk for pg_sys::List {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> TraversalControl {
        if self.type_ != pg_sys::NodeTag::T_List {
            // IntList, OidList, XidList do not contain Node pointers.
            return TraversalControl::Continue;
        }
        let list_ptr = self as *const pg_sys::List as *mut pg_sys::List;
        crate::memcx::current_context(|mcx| unsafe {
            if let Some(list) = crate::list::List::<*mut core::ffi::c_void>::downcast_ptr_in_memcx(list_ptr, mcx) {
                for cell_ptr in list.iter() {
                    if !cell_ptr.is_null() {
                        let node = &*(*cell_ptr as *const pg_sys::Node);
                        if node.walk(visitor).is_break() {
                            return TraversalControl::Break;
                        }
                    }
                }
            }
            TraversalControl::Continue
        })
    }
}

impl PgNodeWalk for pg_sys::CreateStmt {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> TraversalControl {
        if !self.relation.is_null() {
            if visitor.visit_range_var(unsafe { &*self.relation }).is_break() {
                return TraversalControl::Break;
            }
        }
        if !self.tableElts.is_null() {
            if visitor.visit_create_table_elts(unsafe { &*self.tableElts }).is_break() {
                return TraversalControl::Break;
            }
        }
        if !self.inhRelations.is_null() {
            if visitor.visit_create_inh_relations(unsafe { &*self.inhRelations }).is_break() {
                return TraversalControl::Break;
            }
        }
        if !self.constraints.is_null() {
            if visitor.visit_create_constraints(unsafe { &*self.constraints }).is_break() {
                return TraversalControl::Break;
            }
        }
        if !self.options.is_null() {
            if visitor.visit_create_options(unsafe { &*self.options }).is_break() {
                return TraversalControl::Break;
            }
        }
        TraversalControl::Continue
    }
}

impl PgNodeWalk for pg_sys::AlterTableStmt {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> TraversalControl {
        if !self.relation.is_null() {
            if visitor.visit_range_var(unsafe { &*self.relation }).is_break() {
                return TraversalControl::Break;
            }
        }
        if !self.cmds.is_null() {
            if visitor.visit_alter_table_cmds(unsafe { &*self.cmds }).is_break() {
                return TraversalControl::Break;
            }
        }
        TraversalControl::Continue
    }
}

impl PgNodeWalk for pg_sys::AlterTableCmd {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> TraversalControl {
        if !self.def.is_null() {
            if visitor.visit_node(unsafe { &*self.def }).is_break() {
                return TraversalControl::Break;
            }
        }
        TraversalControl::Continue
    }
}

impl PgNodeWalk for pg_sys::DropStmt {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> TraversalControl {
        if !self.objects.is_null() {
            if visitor.visit_drop_objects(unsafe { &*self.objects }).is_break() {
                return TraversalControl::Break;
            }
        }
        TraversalControl::Continue
    }
}

impl PgNodeWalk for pg_sys::TruncateStmt {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> TraversalControl {
        if !self.relations.is_null() {
            if visitor.visit_truncate_relations(unsafe { &*self.relations }).is_break() {
                return TraversalControl::Break;
            }
        }
        TraversalControl::Continue
    }
}

impl PgNodeWalk for pg_sys::IndexStmt {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> TraversalControl {
        if !self.relation.is_null() {
            if visitor.visit_range_var(unsafe { &*self.relation }).is_break() {
                return TraversalControl::Break;
            }
        }
        if !self.indexParams.is_null() {
            if visitor.visit_index_params(unsafe { &*self.indexParams }).is_break() {
                return TraversalControl::Break;
            }
        }
        if !self.indexIncludingParams.is_null() {
            if visitor.visit_index_including_params(unsafe { &*self.indexIncludingParams }).is_break() {
                return TraversalControl::Break;
            }
        }
        if !self.options.is_null() {
            if visitor.visit_index_options(unsafe { &*self.options }).is_break() {
                return TraversalControl::Break;
            }
        }
        if !self.whereClause.is_null() {
            if visitor.visit_node(unsafe { &*self.whereClause }).is_break() {
                return TraversalControl::Break;
            }
        }
        TraversalControl::Continue
    }
}

impl PgNodeWalk for pg_sys::RangeVar {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, _visitor: &mut V) -> TraversalControl {
        TraversalControl::Continue
    }
}

impl PgNodeWalk for pg_sys::RangeTblEntry {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> TraversalControl {
        if !self.subquery.is_null() {
            // RTE_SUBQUERY
            if unsafe { &*self.subquery }.as_node().walk(visitor).is_break() {
                return TraversalControl::Break;
            }
        }
        TraversalControl::Continue
    }
}

impl PgNodeWalk for pg_sys::Query {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> TraversalControl {
        if !self.utilityStmt.is_null() {
            if unsafe { &*self.utilityStmt }.walk(visitor).is_break() {
                return TraversalControl::Break;
            }
        }
        if !self.rtable.is_null() {
            if visitor.visit_list(unsafe { &*self.rtable }).is_break() {
                return TraversalControl::Break;
            }
        }
        // Jointree and other fields omitted for now in manual strategy
        TraversalControl::Continue
    }
}

impl PgNodeWalk for pg_sys::Plan {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> TraversalControl {
        if !self.targetlist.is_null() {
            if visitor.visit_plan_target_list(unsafe { &*self.targetlist }).is_break() {
                return TraversalControl::Break;
            }
        }
        if !self.qual.is_null() {
            if visitor.visit_plan_qual(unsafe { &*self.qual }).is_break() {
                return TraversalControl::Break;
            }
        }
        if !self.lefttree.is_null() {
            if unsafe { &*self.lefttree }.as_node().walk(visitor).is_break() {
                return TraversalControl::Break;
            }
        }
        if !self.righttree.is_null() {
            if unsafe { &*self.righttree }.as_node().walk(visitor).is_break() {
                return TraversalControl::Break;
            }
        }
        if !self.initPlan.is_null() {
            if visitor.visit_list(unsafe { &*self.initPlan }).is_break() {
                return TraversalControl::Break;
            }
        }
        TraversalControl::Continue
    }
}

impl PgNodeWalk for pg_sys::SeqScan {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, _visitor: &mut V) -> TraversalControl {
        TraversalControl::Continue
    }
}

impl PgNodeWalk for pg_sys::ModifyTable {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> TraversalControl {
        // outerPlan(mt) is the source plan
        if !self.plan.lefttree.is_null() {
            if unsafe { &*self.plan.lefttree }.as_node().walk(visitor).is_break() {
                return TraversalControl::Break;
            }
        }
        if !self.resultRelations.is_null() {
            if visitor.visit_modify_table_result_relations(unsafe { &*self.resultRelations }).is_break() {
                return TraversalControl::Break;
            }
        }
        TraversalControl::Continue
    }
}

impl PgNodeWalk for pg_sys::Agg {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> TraversalControl {
        if !self.groupingSets.is_null() {
            if visitor.visit_list(unsafe { &*self.groupingSets }).is_break() {
                return TraversalControl::Break;
            }
        }
        if !self.chain.is_null() {
            if visitor.visit_list(unsafe { &*self.chain }).is_break() {
                return TraversalControl::Break;
            }
        }
        TraversalControl::Continue
    }
}

impl PgNodeWalk for pg_sys::Sort {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, _visitor: &mut V) -> TraversalControl {
        TraversalControl::Continue
    }
}

impl PgNodeWalk for pg_sys::Append {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> TraversalControl {
        if !self.appendplans.is_null() {
            if visitor.visit_append_plans(unsafe { &*self.appendplans }).is_break() {
                return TraversalControl::Break;
            }
        }
        TraversalControl::Continue
    }
}

impl PgNodeWalk for pg_sys::MergeAppend {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> TraversalControl {
        if !self.mergeplans.is_null() {
            if visitor.visit_merge_append_plans(unsafe { &*self.mergeplans }).is_break() {
                return TraversalControl::Break;
            }
        }
        TraversalControl::Continue
    }
}

impl PgNodeWalk for pg_sys::TargetEntry {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> TraversalControl {
        if !self.expr.is_null() {
            if unsafe { &*self.expr }.as_node().walk(visitor).is_break() {
                return TraversalControl::Break;
            }
        }
        TraversalControl::Continue
    }
}

impl PgNodeWalk for pg_sys::PlannedStmt {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> TraversalControl {
        if self.commandType == pg_sys::CmdType::CMD_UTILITY && !self.utilityStmt.is_null() {
            unsafe { &*self.utilityStmt }.walk(visitor)
        } else {
            if !self.planTree.is_null() {
                if visitor.visit_planned_stmt_plan_tree(unsafe { &*self.planTree }).is_break() {
                    return TraversalControl::Break;
                }
            }
            if !self.rtable.is_null() {
                if visitor.visit_planned_stmt_rtable(unsafe { &*self.rtable }).is_break() {
                    return TraversalControl::Break;
                }
            }
            if !self.subplans.is_null() {
                if visitor.visit_planned_stmt_subplans(unsafe { &*self.subplans }).is_break() {
                    return TraversalControl::Break;
                }
            }
            TraversalControl::Continue
        }
    }
}
