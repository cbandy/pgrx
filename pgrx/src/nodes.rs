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
pub enum Traversal {
    Continue,
    Break,
}

impl Traversal {
    #[inline]
    pub fn is_break(&self) -> bool {
        matches!(self, Traversal::Break)
    }
}

pub trait AsPgNode {
    fn as_node(&self) -> &pg_sys::Node;
    fn as_node_mut(&mut self) -> &mut pg_sys::Node;
}

macro_rules! as_pg_node {
    ($struct_name:ident) => {
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
    };
}

pub trait HasNodeTag: AsPgNode {
    const NODE_TAGS: &'static [pg_sys::NodeTag];
}

macro_rules! has_node_tag {
    ($struct_name:ident, [$($tag_variant:ident),+]) => {
        as_pg_node!($struct_name);
        impl HasNodeTag for pg_sys::$struct_name {
            const NODE_TAGS: &'static [pg_sys::NodeTag] = &[
                $(pg_sys::NodeTag::$tag_variant),+
            ];
        }
    };
    ($struct_name:ident, $tag_variant:ident) => {
        as_pg_node!($struct_name);
        impl HasNodeTag for pg_sys::$struct_name {
            const NODE_TAGS: &'static [pg_sys::NodeTag] = &[pg_sys::NodeTag::$tag_variant];
        }
    };
}

has_node_tag!(CreateStmt, T_CreateStmt);
has_node_tag!(AlterTableStmt, T_AlterTableStmt);
has_node_tag!(AlterTableCmd, T_AlterTableCmd);
has_node_tag!(DropStmt, T_DropStmt);
has_node_tag!(RangeVar, T_RangeVar);
has_node_tag!(List, [T_List, T_IntList, T_OidList, T_XidList]);
has_node_tag!(TruncateStmt, T_TruncateStmt);
has_node_tag!(IndexStmt, T_IndexStmt);
has_node_tag!(RangeTblEntry, T_RangeTblEntry);
has_node_tag!(Var, T_Var);
has_node_tag!(Const, T_Const);
has_node_tag!(OpExpr, T_OpExpr);
has_node_tag!(FuncExpr, T_FuncExpr);
has_node_tag!(BoolExpr, T_BoolExpr);
has_node_tag!(SubLink, T_SubLink);
has_node_tag!(SubPlan, T_SubPlan);
has_node_tag!(ScalarArrayOpExpr, T_ScalarArrayOpExpr);
has_node_tag!(SeqScan, T_SeqScan);
has_node_tag!(ModifyTable, T_ModifyTable);
has_node_tag!(Agg, T_Agg);
has_node_tag!(Sort, T_Sort);
has_node_tag!(TargetEntry, T_TargetEntry);
has_node_tag!(Append, T_Append);
has_node_tag!(MergeAppend, T_MergeAppend);
has_node_tag!(Query, T_Query);
has_node_tag!(FromExpr, T_FromExpr);
has_node_tag!(Result, T_Result);
has_node_tag!(ProjectSet, T_ProjectSet);
has_node_tag!(IndexScan, T_IndexScan);
has_node_tag!(IndexOnlyScan, T_IndexOnlyScan);
has_node_tag!(BitmapIndexScan, T_BitmapIndexScan);
has_node_tag!(BitmapHeapScan, T_BitmapHeapScan);
has_node_tag!(TidScan, T_TidScan);
has_node_tag!(TidRangeScan, T_TidRangeScan);
has_node_tag!(PartitionPruneInfo, T_PartitionPruneInfo);
has_node_tag!(PlanRowMark, T_PlanRowMark);
has_node_tag!(RTEPermissionInfo, T_RTEPermissionInfo);
has_node_tag!(AppendRelInfo, T_AppendRelInfo);
has_node_tag!(PlanInvalItem, T_PlanInvalItem);
has_node_tag!(NestLoop, T_NestLoop);
has_node_tag!(MergeJoin, T_MergeJoin);
has_node_tag!(HashJoin, T_HashJoin);
has_node_tag!(Material, T_Material);
has_node_tag!(Limit, T_Limit);
has_node_tag!(BitmapAnd, T_BitmapAnd);
has_node_tag!(BitmapOr, T_BitmapOr);
has_node_tag!(PartitionedRelPruneInfo, T_PartitionedRelPruneInfo);
has_node_tag!(PartitionPruneStepOp, T_PartitionPruneStepOp);
has_node_tag!(PartitionPruneStepCombine, T_PartitionPruneStepCombine);
has_node_tag!(RecursiveUnion, T_RecursiveUnion);
has_node_tag!(ValuesScan, T_ValuesScan);
has_node_tag!(CteScan, T_CteScan);
has_node_tag!(Gather, T_Gather);
has_node_tag!(GatherMerge, T_GatherMerge);
has_node_tag!(Hash, T_Hash);
has_node_tag!(LockRows, T_LockRows);
has_node_tag!(WindowAgg, T_WindowAgg);
has_node_tag!(Unique, T_Unique);
has_node_tag!(SetOp, T_SetOp);

as_pg_node!(Plan);
as_pg_node!(Scan);
as_pg_node!(Expr);
as_pg_node!(Join);
as_pg_node!(PartitionPruneStep);

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
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> Traversal;
}

macro_rules! impl_pg_node_walk {
    ($struct_name:ident) => {
        impl PgNodeWalk for pg_sys::$struct_name {
            #[inline]
            fn walk<V: PlannedStmtVisitor + ?Sized>(&self, _visitor: &mut V) -> Traversal {
                Traversal::Continue
            }
        }
    };
}

pub trait PlannedStmtVisitor {
    // Generic catch-all — defaults to Continue (no-op)
    fn visit_node(&mut self, _node: &pg_sys::Node) -> Traversal {
        Traversal::Continue
    }

    // Generic list catch-all — used only from Node::walk for unrecognised lists
    fn visit_list(&mut self, list: &pg_sys::List) -> Traversal {
        if self.visit_node(list.as_node()).is_break() {
            return Traversal::Break;
        }
        list.walk(self)
    }

    // Statement-level hooks
    fn visit_create_stmt(&mut self, stmt: &pg_sys::CreateStmt) -> Traversal {
        if self.visit_node(stmt.as_node()).is_break() { return Traversal::Break; }
        stmt.walk(self)
    }

    fn visit_alter_table_stmt(&mut self, stmt: &pg_sys::AlterTableStmt) -> Traversal {
        if self.visit_node(stmt.as_node()).is_break() { return Traversal::Break; }
        stmt.walk(self)
    }

    fn visit_alter_table_cmd(&mut self, cmd: &pg_sys::AlterTableCmd) -> Traversal {
        if self.visit_node(cmd.as_node()).is_break() { return Traversal::Break; }
        cmd.walk(self)
    }

    fn visit_drop_stmt(&mut self, stmt: &pg_sys::DropStmt) -> Traversal {
        if self.visit_node(stmt.as_node()).is_break() { return Traversal::Break; }
        stmt.walk(self)
    }

    fn visit_truncate_stmt(&mut self, stmt: &pg_sys::TruncateStmt) -> Traversal {
        if self.visit_node(stmt.as_node()).is_break() { return Traversal::Break; }
        stmt.walk(self)
    }

    fn visit_index_stmt(&mut self, stmt: &pg_sys::IndexStmt) -> Traversal {
        if self.visit_node(stmt.as_node()).is_break() { return Traversal::Break; }
        stmt.walk(self)
    }

    // Shared leaf hook
    fn visit_range_var(&mut self, range_var: &pg_sys::RangeVar) -> Traversal {
        if self.visit_node(range_var.as_node()).is_break() { return Traversal::Break; }
        range_var.walk(self)
    }

    // CreateStmt field hooks
    fn visit_create_table_elts(&mut self, list: &pg_sys::List) -> Traversal {
        self.visit_list(list)
    }
    fn visit_create_inh_relations(&mut self, list: &pg_sys::List) -> Traversal {
        self.visit_list(list)
    }
    fn visit_create_constraints(&mut self, list: &pg_sys::List) -> Traversal {
        self.visit_list(list)
    }
    fn visit_create_options(&mut self, list: &pg_sys::List) -> Traversal {
        self.visit_list(list)
    }

    // AlterTableStmt field hook
    fn visit_alter_table_cmds(&mut self, list: &pg_sys::List) -> Traversal {
        self.visit_list(list)
    }

    // DropStmt field hook
    fn visit_drop_objects(&mut self, list: &pg_sys::List) -> Traversal {
        self.visit_list(list)
    }

    // TruncateStmt field hook
    fn visit_truncate_relations(&mut self, list: &pg_sys::List) -> Traversal {
        self.visit_list(list)
    }

    // ValuesScan field hooks
    fn visit_values_scan_lists(&mut self, list: &pg_sys::List) -> Traversal {
        self.visit_list(list)
    }

    // IndexStmt field hooks — each is distinct, allowing unambiguous overrides
    fn visit_index_params(&mut self, list: &pg_sys::List) -> Traversal {
        self.visit_list(list)
    }
    fn visit_index_including_params(&mut self, list: &pg_sys::List) -> Traversal {
        self.visit_list(list)
    }
    fn visit_index_options(&mut self, list: &pg_sys::List) -> Traversal {
        self.visit_list(list)
    }

    // DML / Plan hooks
    fn visit_plan(&mut self, plan: &pg_sys::Plan) -> Traversal {
        self.visit_node(plan.as_node())
    }

    fn visit_result(&mut self, res: &pg_sys::Result) -> Traversal {
        if self.visit_plan(&res.plan).is_break() { return Traversal::Break; }
        res.walk(self)
    }

    fn visit_project_set(&mut self, ps: &pg_sys::ProjectSet) -> Traversal {
        if self.visit_plan(&ps.plan).is_break() { return Traversal::Break; }
        ps.walk(self)
    }

    fn visit_scan(&mut self, scan: &pg_sys::Scan) -> Traversal {
        self.visit_plan(&scan.plan)
    }

    fn visit_seq_scan(&mut self, scan: &pg_sys::SeqScan) -> Traversal {
        if self.visit_scan(&scan.scan).is_break() { return Traversal::Break; }
        scan.walk(self)
    }

    fn visit_index_scan(&mut self, scan: &pg_sys::IndexScan) -> Traversal {
        if self.visit_scan(&scan.scan).is_break() { return Traversal::Break; }
        scan.walk(self)
    }

    fn visit_index_only_scan(&mut self, scan: &pg_sys::IndexOnlyScan) -> Traversal {
        if self.visit_scan(&scan.scan).is_break() { return Traversal::Break; }
        scan.walk(self)
    }

    fn visit_bitmap_index_scan(&mut self, scan: &pg_sys::BitmapIndexScan) -> Traversal {
        if self.visit_scan(&scan.scan).is_break() { return Traversal::Break; }
        scan.walk(self)
    }

    fn visit_bitmap_heap_scan(&mut self, scan: &pg_sys::BitmapHeapScan) -> Traversal {
        if self.visit_scan(&scan.scan).is_break() { return Traversal::Break; }
        scan.walk(self)
    }

    fn visit_tid_scan(&mut self, scan: &pg_sys::TidScan) -> Traversal {
        if self.visit_scan(&scan.scan).is_break() { return Traversal::Break; }
        scan.walk(self)
    }

    fn visit_tid_range_scan(&mut self, scan: &pg_sys::TidRangeScan) -> Traversal {
        if self.visit_scan(&scan.scan).is_break() { return Traversal::Break; }
        scan.walk(self)
    }

    fn visit_join(&mut self, join: &pg_sys::Join) -> Traversal {
        self.visit_plan(&join.plan)
    }

    fn visit_nest_loop(&mut self, join: &pg_sys::NestLoop) -> Traversal {
        if self.visit_join(&join.join).is_break() { return Traversal::Break; }
        join.walk(self)
    }

    fn visit_merge_join(&mut self, join: &pg_sys::MergeJoin) -> Traversal {
        if self.visit_join(&join.join).is_break() { return Traversal::Break; }
        join.walk(self)
    }

    fn visit_hash_join(&mut self, join: &pg_sys::HashJoin) -> Traversal {
        if self.visit_join(&join.join).is_break() { return Traversal::Break; }
        join.walk(self)
    }

    fn visit_material(&mut self, mat: &pg_sys::Material) -> Traversal {
        if self.visit_plan(&mat.plan).is_break() { return Traversal::Break; }
        mat.walk(self)
    }

    fn visit_limit(&mut self, limit: &pg_sys::Limit) -> Traversal {
        if self.visit_plan(&limit.plan).is_break() { return Traversal::Break; }
        limit.walk(self)
    }

    fn visit_modify_table(&mut self, mt: &pg_sys::ModifyTable) -> Traversal {
        if self.visit_plan(&mt.plan).is_break() { return Traversal::Break; }
        mt.walk(self)
    }

    fn visit_agg(&mut self, agg: &pg_sys::Agg) -> Traversal {
        if self.visit_plan(&agg.plan).is_break() { return Traversal::Break; }
        agg.walk(self)
    }

    fn visit_sort(&mut self, sort: &pg_sys::Sort) -> Traversal {
        if self.visit_plan(&sort.plan).is_break() { return Traversal::Break; }
        sort.walk(self)
    }

    fn visit_append(&mut self, append: &pg_sys::Append) -> Traversal {
        if self.visit_plan(&append.plan).is_break() { return Traversal::Break; }
        append.walk(self)
    }

    fn visit_merge_append(&mut self, ma: &pg_sys::MergeAppend) -> Traversal {
        if self.visit_plan(&ma.plan).is_break() { return Traversal::Break; }
        ma.walk(self)
    }

    fn visit_recursive_union(&mut self, join: &pg_sys::RecursiveUnion) -> Traversal {
        if self.visit_plan(&join.plan).is_break() { return Traversal::Break; }
        join.walk(self)
    }

    fn visit_values_scan(&mut self, scan: &pg_sys::ValuesScan) -> Traversal {
        if self.visit_scan(&scan.scan).is_break() { return Traversal::Break; }
        scan.walk(self)
    }

    fn visit_cte_scan(&mut self, scan: &pg_sys::CteScan) -> Traversal {
        if self.visit_scan(&scan.scan).is_break() { return Traversal::Break; }
        scan.walk(self)
    }

    fn visit_gather(&mut self, gather: &pg_sys::Gather) -> Traversal {
        if self.visit_plan(&gather.plan).is_break() { return Traversal::Break; }
        gather.walk(self)
    }

    fn visit_gather_merge(&mut self, gather: &pg_sys::GatherMerge) -> Traversal {
        if self.visit_plan(&gather.plan).is_break() { return Traversal::Break; }
        gather.walk(self)
    }

    fn visit_hash(&mut self, hash: &pg_sys::Hash) -> Traversal {
        if self.visit_plan(&hash.plan).is_break() { return Traversal::Break; }
        hash.walk(self)
    }

    fn visit_lock_rows(&mut self, lock: &pg_sys::LockRows) -> Traversal {
        if self.visit_plan(&lock.plan).is_break() { return Traversal::Break; }
        lock.walk(self)
    }

    fn visit_window_agg(&mut self, agg: &pg_sys::WindowAgg) -> Traversal {
        if self.visit_plan(&agg.plan).is_break() { return Traversal::Break; }
        agg.walk(self)
    }

    fn visit_unique(&mut self, unique: &pg_sys::Unique) -> Traversal {
        if self.visit_plan(&unique.plan).is_break() { return Traversal::Break; }
        unique.walk(self)
    }

    fn visit_set_op(&mut self, setop: &pg_sys::SetOp) -> Traversal {
        if self.visit_plan(&setop.plan).is_break() { return Traversal::Break; }
        setop.walk(self)
    }

    fn visit_bitmap_and(&mut self, ba: &pg_sys::BitmapAnd) -> Traversal {
        if self.visit_plan(&ba.plan).is_break() { return Traversal::Break; }
        ba.walk(self)
    }

    fn visit_bitmap_or(&mut self, bo: &pg_sys::BitmapOr) -> Traversal {
        if self.visit_plan(&bo.plan).is_break() { return Traversal::Break; }
        bo.walk(self)
    }

    fn visit_range_tbl_entry(&mut self, rte: &pg_sys::RangeTblEntry) -> Traversal {
        if self.visit_node(rte.as_node()).is_break() { return Traversal::Break; }
        rte.walk(self)
    }

    fn visit_target_entry(&mut self, te: &pg_sys::TargetEntry) -> Traversal {
        if self.visit_node(te.as_node()).is_break() { return Traversal::Break; }
        te.walk(self)
    }

    fn visit_query(&mut self, query: &pg_sys::Query) -> Traversal {
        if self.visit_node(query.as_node()).is_break() { return Traversal::Break; }
        query.walk(self)
    }

    fn visit_partition_prune_info(&mut self, info: &pg_sys::PartitionPruneInfo) -> Traversal {
        if self.visit_node(info.as_node()).is_break() { return Traversal::Break; }
        info.walk(self)
    }

    fn visit_partitioned_rel_prune_info(&mut self, info: &pg_sys::PartitionedRelPruneInfo) -> Traversal {
        if self.visit_node(info.as_node()).is_break() { return Traversal::Break; }
        info.walk(self)
    }

    fn visit_partition_prune_step(&mut self, step: &pg_sys::PartitionPruneStep) -> Traversal {
        self.visit_node(step.as_node())
    }

    fn visit_partition_prune_step_op(&mut self, step: &pg_sys::PartitionPruneStepOp) -> Traversal {
        if self.visit_partition_prune_step(&step.step).is_break() { return Traversal::Break; }
        step.walk(self)
    }

    fn visit_partition_prune_step_combine(&mut self, step: &pg_sys::PartitionPruneStepCombine) -> Traversal {
        if self.visit_partition_prune_step(&step.step).is_break() { return Traversal::Break; }
        step.walk(self)
    }

    fn visit_plan_row_mark(&mut self, mark: &pg_sys::PlanRowMark) -> Traversal {
        if self.visit_node(mark.as_node()).is_break() { return Traversal::Break; }
        mark.walk(self)
    }

    fn visit_rte_permission_info(&mut self, info: &pg_sys::RTEPermissionInfo) -> Traversal {
        if self.visit_node(info.as_node()).is_break() { return Traversal::Break; }
        info.walk(self)
    }

    fn visit_append_rel_info(&mut self, info: &pg_sys::AppendRelInfo) -> Traversal {
        if self.visit_node(info.as_node()).is_break() { return Traversal::Break; }
        info.walk(self)
    }

    fn visit_plan_inval_item(&mut self, item: &pg_sys::PlanInvalItem) -> Traversal {
        if self.visit_node(item.as_node()).is_break() { return Traversal::Break; }
        item.walk(self)
    }

    // PlannedStmt field hooks
    fn visit_planned_stmt_rtable(&mut self, list: &pg_sys::List) -> Traversal {
        self.visit_list(list)
    }
    fn visit_planned_stmt_subplans(&mut self, list: &pg_sys::List) -> Traversal {
        self.visit_list(list)
    }
    fn visit_planned_stmt_plan_tree(&mut self, plan: &pg_sys::Plan) -> Traversal {
        plan.as_node().walk(self)
    }
    fn visit_planned_stmt_perm_infos(&mut self, list: &pg_sys::List) -> Traversal {
        self.visit_list(list)
    }
    fn visit_planned_stmt_row_marks(&mut self, list: &pg_sys::List) -> Traversal {
        self.visit_list(list)
    }
    fn visit_planned_stmt_inval_items(&mut self, list: &pg_sys::List) -> Traversal {
        self.visit_list(list)
    }
    fn visit_planned_stmt_append_relations(&mut self, list: &pg_sys::List) -> Traversal {
        self.visit_list(list)
    }
    fn visit_planned_stmt_part_prune_infos(&mut self, list: &pg_sys::List) -> Traversal {
        self.visit_list(list)
    }

    // Expr hooks
    fn visit_var(&mut self, var: &pg_sys::Var) -> Traversal {
        self.visit_node(var.as_node())
    }
    fn visit_const(&mut self, con: &pg_sys::Const) -> Traversal {
        self.visit_node(con.as_node())
    }
    fn visit_op_expr(&mut self, expr: &pg_sys::OpExpr) -> Traversal {
        if self.visit_node(expr.as_node()).is_break() { return Traversal::Break; }
        expr.walk(self)
    }
    fn visit_func_expr(&mut self, expr: &pg_sys::FuncExpr) -> Traversal {
        if self.visit_node(expr.as_node()).is_break() { return Traversal::Break; }
        expr.walk(self)
    }
    fn visit_bool_expr(&mut self, expr: &pg_sys::BoolExpr) -> Traversal {
        if self.visit_node(expr.as_node()).is_break() { return Traversal::Break; }
        expr.walk(self)
    }
    fn visit_sub_link(&mut self, expr: &pg_sys::SubLink) -> Traversal {
        if self.visit_node(expr.as_node()).is_break() { return Traversal::Break; }
        expr.walk(self)
    }
    fn visit_sub_plan(&mut self, expr: &pg_sys::SubPlan) -> Traversal {
        if self.visit_node(expr.as_node()).is_break() { return Traversal::Break; }
        expr.walk(self)
    }
    fn visit_scalar_array_op_expr(&mut self, expr: &pg_sys::ScalarArrayOpExpr) -> Traversal {
        if self.visit_node(expr.as_node()).is_break() { return Traversal::Break; }
        expr.walk(self)
    }

    // Query field hooks
    fn visit_query_cte_list(&mut self, list: &pg_sys::List) -> Traversal {
        self.visit_list(list)
    }
    fn visit_query_rtable(&mut self, list: &pg_sys::List) -> Traversal {
        self.visit_list(list)
    }
    fn visit_query_target_list(&mut self, list: &pg_sys::List) -> Traversal {
        self.visit_list(list)
    }
    fn visit_query_returning_list(&mut self, list: &pg_sys::List) -> Traversal {
        self.visit_list(list)
    }

    // Plan field hooks
    fn visit_plan_target_list(&mut self, list: &pg_sys::List) -> Traversal {
        self.visit_list(list)
    }
    fn visit_plan_qual(&mut self, list: &pg_sys::List) -> Traversal {
        self.visit_list(list)
    }

    // Join field hooks
    fn visit_join_qual(&mut self, list: &pg_sys::List) -> Traversal {
        self.visit_list(list)
    }

    // MergeJoin field hooks
    fn visit_merge_join_clauses(&mut self, list: &pg_sys::List) -> Traversal {
        self.visit_list(list)
    }

    // HashJoin field hooks
    fn visit_hash_join_clauses(&mut self, list: &pg_sys::List) -> Traversal {
        self.visit_list(list)
    }

    // ModifyTable field hooks
    fn visit_modify_table_result_relations(&mut self, list: &pg_sys::List) -> Traversal {
        self.visit_list(list)
    }

    // Append field hooks
    fn visit_append_plans(&mut self, list: &pg_sys::List) -> Traversal {
        self.visit_list(list)
    }

    // MergeAppend field hooks
    fn visit_merge_append_plans(&mut self, list: &pg_sys::List) -> Traversal {
        self.visit_list(list)
    }

    // IndexScan field hooks
    fn visit_index_scan_quals(&mut self, list: &pg_sys::List) -> Traversal {
        self.visit_list(list)
    }
    fn visit_index_scan_orderby(&mut self, list: &pg_sys::List) -> Traversal {
        self.visit_list(list)
    }

    // PartitionPruneInfo field hooks
    fn visit_partition_prune_infos(&mut self, list: &pg_sys::List) -> Traversal {
        self.visit_list(list)
    }

    // PartitionedRelPruneInfo field hooks
    fn visit_pruning_steps(&mut self, list: &pg_sys::List) -> Traversal {
        self.visit_list(list)
    }

    // BitmapAnd/Or field hooks
    fn visit_bitmap_plans(&mut self, list: &pg_sys::List) -> Traversal {
        self.visit_list(list)
    }

    // Hash field hooks
    fn visit_hash_keys(&mut self, list: &pg_sys::List) -> Traversal {
        self.visit_list(list)
    }

    // LockRows field hooks
    fn visit_lock_rows_marks(&mut self, list: &pg_sys::List) -> Traversal {
        self.visit_list(list)
    }
}

impl PgNodeWalk for pg_sys::Node {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> Traversal {
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
            pg_sys::NodeTag::T_FromExpr => {
                let from = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::FromExpr) };
                if visitor.visit_node(from.as_node()).is_break() { return Traversal::Break; }
                from.walk(visitor)
            }
            pg_sys::NodeTag::T_Result => {
                let res = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::Result) };
                visitor.visit_result(res)
            }
            pg_sys::NodeTag::T_ProjectSet => {
                let ps = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::ProjectSet) };
                visitor.visit_project_set(ps)
            }
            pg_sys::NodeTag::T_IndexScan => {
                let scan = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::IndexScan) };
                visitor.visit_index_scan(scan)
            }
            pg_sys::NodeTag::T_IndexOnlyScan => {
                let scan = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::IndexOnlyScan) };
                visitor.visit_index_only_scan(scan)
            }
            pg_sys::NodeTag::T_BitmapIndexScan => {
                let scan = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::BitmapIndexScan) };
                visitor.visit_bitmap_index_scan(scan)
            }
            pg_sys::NodeTag::T_BitmapHeapScan => {
                let scan = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::BitmapHeapScan) };
                visitor.visit_bitmap_heap_scan(scan)
            }
            pg_sys::NodeTag::T_TidScan => {
                let scan = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::TidScan) };
                visitor.visit_tid_scan(scan)
            }
            pg_sys::NodeTag::T_TidRangeScan => {
                let scan = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::TidRangeScan) };
                visitor.visit_tid_range_scan(scan)
            }
            pg_sys::NodeTag::T_PartitionPruneInfo => {
                let info = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::PartitionPruneInfo) };
                visitor.visit_partition_prune_info(info)
            }
            pg_sys::NodeTag::T_PlanRowMark => {
                let mark = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::PlanRowMark) };
                visitor.visit_plan_row_mark(mark)
            }
            pg_sys::NodeTag::T_RTEPermissionInfo => {
                let info = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::RTEPermissionInfo) };
                visitor.visit_rte_permission_info(info)
            }
            pg_sys::NodeTag::T_AppendRelInfo => {
                let info = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::AppendRelInfo) };
                visitor.visit_append_rel_info(info)
            }
            pg_sys::NodeTag::T_PlanInvalItem => {
                let item = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::PlanInvalItem) };
                visitor.visit_plan_inval_item(item)
            }
            pg_sys::NodeTag::T_NestLoop => {
                let join = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::NestLoop) };
                visitor.visit_nest_loop(join)
            }
            pg_sys::NodeTag::T_MergeJoin => {
                let join = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::MergeJoin) };
                visitor.visit_merge_join(join)
            }
            pg_sys::NodeTag::T_HashJoin => {
                let join = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::HashJoin) };
                visitor.visit_hash_join(join)
            }
            pg_sys::NodeTag::T_Material => {
                let mat = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::Material) };
                visitor.visit_material(mat)
            }
            pg_sys::NodeTag::T_Limit => {
                let limit = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::Limit) };
                visitor.visit_limit(limit)
            }
            pg_sys::NodeTag::T_BitmapAnd => {
                let ba = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::BitmapAnd) };
                visitor.visit_bitmap_and(ba)
            }
            pg_sys::NodeTag::T_BitmapOr => {
                let bo = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::BitmapOr) };
                visitor.visit_bitmap_or(bo)
            }
            pg_sys::NodeTag::T_PartitionedRelPruneInfo => {
                let info = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::PartitionedRelPruneInfo) };
                visitor.visit_partitioned_rel_prune_info(info)
            }
            pg_sys::NodeTag::T_PartitionPruneStepOp => {
                let step = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::PartitionPruneStepOp) };
                visitor.visit_partition_prune_step_op(step)
            }
            pg_sys::NodeTag::T_PartitionPruneStepCombine => {
                let step = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::PartitionPruneStepCombine) };
                visitor.visit_partition_prune_step_combine(step)
            }
            pg_sys::NodeTag::T_RecursiveUnion => {
                let union = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::RecursiveUnion) };
                visitor.visit_recursive_union(union)
            }
            pg_sys::NodeTag::T_ValuesScan => {
                let scan = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::ValuesScan) };
                visitor.visit_values_scan(scan)
            }
            pg_sys::NodeTag::T_CteScan => {
                let scan = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::CteScan) };
                visitor.visit_cte_scan(scan)
            }
            pg_sys::NodeTag::T_Gather => {
                let gather = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::Gather) };
                visitor.visit_gather(gather)
            }
            pg_sys::NodeTag::T_GatherMerge => {
                let gather = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::GatherMerge) };
                visitor.visit_gather_merge(gather)
            }
            pg_sys::NodeTag::T_Hash => {
                let hash = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::Hash) };
                visitor.visit_hash(hash)
            }
            pg_sys::NodeTag::T_LockRows => {
                let lock = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::LockRows) };
                visitor.visit_lock_rows(lock)
            }
            pg_sys::NodeTag::T_WindowAgg => {
                let agg = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::WindowAgg) };
                visitor.visit_window_agg(agg)
            }
            pg_sys::NodeTag::T_Unique => {
                let unique = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::Unique) };
                visitor.visit_unique(unique)
            }
            pg_sys::NodeTag::T_SetOp => {
                let setop = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::SetOp) };
                visitor.visit_set_op(setop)
            }
            pg_sys::NodeTag::T_Var => {
                let var = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::Var) };
                visitor.visit_var(var)
            }
            pg_sys::NodeTag::T_Const => {
                let con = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::Const) };
                visitor.visit_const(con)
            }
            pg_sys::NodeTag::T_OpExpr => {
                let expr = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::OpExpr) };
                visitor.visit_op_expr(expr)
            }
            pg_sys::NodeTag::T_FuncExpr => {
                let expr = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::FuncExpr) };
                visitor.visit_func_expr(expr)
            }
            pg_sys::NodeTag::T_BoolExpr => {
                let expr = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::BoolExpr) };
                visitor.visit_bool_expr(expr)
            }
            pg_sys::NodeTag::T_SubLink => {
                let expr = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::SubLink) };
                visitor.visit_sub_link(expr)
            }
            pg_sys::NodeTag::T_SubPlan => {
                let expr = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::SubPlan) };
                visitor.visit_sub_plan(expr)
            }
            pg_sys::NodeTag::T_ScalarArrayOpExpr => {
                let expr = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::ScalarArrayOpExpr) };
                visitor.visit_scalar_array_op_expr(expr)
            }
            _ => visitor.visit_node(self),
        }
    }
}

impl PgNodeWalk for pg_sys::List {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> Traversal {
        if self.type_ != pg_sys::NodeTag::T_List {
            // IntList, OidList, XidList do not contain Node pointers.
            return Traversal::Continue;
        }
        let list_ptr = self as *const pg_sys::List as *mut pg_sys::List;
        crate::memcx::current_context(|mcx| unsafe {
            if let Some(list) = crate::list::List::<*mut core::ffi::c_void>::downcast_ptr_in_memcx(list_ptr, mcx) {
                for cell_ptr in list.iter() {
                    if !cell_ptr.is_null() {
                        let node = &*(*cell_ptr as *const pg_sys::Node);
                        if node.walk(visitor).is_break() {
                            return Traversal::Break;
                        }
                    }
                }
            }
            Traversal::Continue
        })
    }
}

impl PgNodeWalk for pg_sys::CreateStmt {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> Traversal {
        if !self.relation.is_null() {
            if visitor.visit_range_var(unsafe { &*self.relation }).is_break() {
                return Traversal::Break;
            }
        }
        if !self.tableElts.is_null() {
            if visitor.visit_create_table_elts(unsafe { &*self.tableElts }).is_break() {
                return Traversal::Break;
            }
        }
        if !self.inhRelations.is_null() {
            if visitor.visit_create_inh_relations(unsafe { &*self.inhRelations }).is_break() {
                return Traversal::Break;
            }
        }
        if !self.constraints.is_null() {
            if visitor.visit_create_constraints(unsafe { &*self.constraints }).is_break() {
                return Traversal::Break;
            }
        }
        if !self.options.is_null() {
            if visitor.visit_create_options(unsafe { &*self.options }).is_break() {
                return Traversal::Break;
            }
        }
        Traversal::Continue
    }
}

impl PgNodeWalk for pg_sys::AlterTableStmt {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> Traversal {
        if !self.relation.is_null() {
            if visitor.visit_range_var(unsafe { &*self.relation }).is_break() {
                return Traversal::Break;
            }
        }
        if !self.cmds.is_null() {
            if visitor.visit_alter_table_cmds(unsafe { &*self.cmds }).is_break() {
                return Traversal::Break;
            }
        }
        Traversal::Continue
    }
}

impl PgNodeWalk for pg_sys::AlterTableCmd {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> Traversal {
        if !self.def.is_null() {
            if visitor.visit_node(unsafe { &*self.def }).is_break() {
                return Traversal::Break;
            }
        }
        Traversal::Continue
    }
}

impl PgNodeWalk for pg_sys::DropStmt {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> Traversal {
        if !self.objects.is_null() {
            if visitor.visit_drop_objects(unsafe { &*self.objects }).is_break() {
                return Traversal::Break;
            }
        }
        Traversal::Continue
    }
}

impl PgNodeWalk for pg_sys::TruncateStmt {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> Traversal {
        if !self.relations.is_null() {
            if visitor.visit_truncate_relations(unsafe { &*self.relations }).is_break() {
                return Traversal::Break;
            }
        }
        Traversal::Continue
    }
}

impl PgNodeWalk for pg_sys::IndexStmt {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> Traversal {
        if !self.relation.is_null() {
            if visitor.visit_range_var(unsafe { &*self.relation }).is_break() {
                return Traversal::Break;
            }
        }
        if !self.indexParams.is_null() {
            if visitor.visit_index_params(unsafe { &*self.indexParams }).is_break() {
                return Traversal::Break;
            }
        }
        if !self.indexIncludingParams.is_null() {
            if visitor.visit_index_including_params(unsafe { &*self.indexIncludingParams }).is_break() {
                return Traversal::Break;
            }
        }
        if !self.options.is_null() {
            if visitor.visit_index_options(unsafe { &*self.options }).is_break() {
                return Traversal::Break;
            }
        }
        if !self.whereClause.is_null() {
            if visitor.visit_node(unsafe { &*self.whereClause }).is_break() {
                return Traversal::Break;
            }
        }
        Traversal::Continue
    }
}

impl_pg_node_walk!(RangeVar);

impl PgNodeWalk for pg_sys::RangeTblEntry {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> Traversal {
        if !self.subquery.is_null() {
            // RTE_SUBQUERY
            if unsafe { &*self.subquery }.as_node().walk(visitor).is_break() {
                return Traversal::Break;
            }
        }
        Traversal::Continue
    }
}

impl PgNodeWalk for pg_sys::Query {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> Traversal {
        if !self.utilityStmt.is_null() {
            if unsafe { &*self.utilityStmt }.walk(visitor).is_break() {
                return Traversal::Break;
            }
        }
        if !self.cteList.is_null() {
            if visitor.visit_query_cte_list(unsafe { &*self.cteList }).is_break() {
                return Traversal::Break;
            }
        }
        if !self.rtable.is_null() {
            if visitor.visit_query_rtable(unsafe { &*self.rtable }).is_break() {
                return Traversal::Break;
            }
        }
        if !self.jointree.is_null() {
            if unsafe { &*self.jointree }.as_node().walk(visitor).is_break() {
                return Traversal::Break;
            }
        }
        if !self.targetList.is_null() {
            if visitor.visit_query_target_list(unsafe { &*self.targetList }).is_break() {
                return Traversal::Break;
            }
        }
        if !self.returningList.is_null() {
            if visitor.visit_query_returning_list(unsafe { &*self.returningList }).is_break() {
                return Traversal::Break;
            }
        }
        Traversal::Continue
    }
}

impl PgNodeWalk for pg_sys::FromExpr {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> Traversal {
        if !self.fromlist.is_null() {
            if visitor.visit_list(unsafe { &*self.fromlist }).is_break() {
                return Traversal::Break;
            }
        }
        if !self.quals.is_null() {
            if unsafe { &*self.quals }.walk(visitor).is_break() {
                return Traversal::Break;
            }
        }
        Traversal::Continue
    }
}

impl PgNodeWalk for pg_sys::Plan {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> Traversal {
        if !self.targetlist.is_null() {
            if visitor.visit_plan_target_list(unsafe { &*self.targetlist }).is_break() {
                return Traversal::Break;
            }
        }
        if !self.qual.is_null() {
            if visitor.visit_plan_qual(unsafe { &*self.qual }).is_break() {
                return Traversal::Break;
            }
        }
        if !self.lefttree.is_null() {
            if unsafe { &*self.lefttree }.as_node().walk(visitor).is_break() {
                return Traversal::Break;
            }
        }
        if !self.righttree.is_null() {
            if unsafe { &*self.righttree }.as_node().walk(visitor).is_break() {
                return Traversal::Break;
            }
        }
        if !self.initPlan.is_null() {
            if visitor.visit_list(unsafe { &*self.initPlan }).is_break() {
                return Traversal::Break;
            }
        }
        Traversal::Continue
    }
}

impl PgNodeWalk for pg_sys::Result {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> Traversal {
        if !self.resconstantqual.is_null() {
             if unsafe { &*self.resconstantqual }.walk(visitor).is_break() {
                 return Traversal::Break;
             }
        }
        Traversal::Continue
    }
}

impl_pg_node_walk!(ProjectSet);
impl_pg_node_walk!(Scan);
impl_pg_node_walk!(SeqScan);

impl PgNodeWalk for pg_sys::IndexScan {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> Traversal {
        if !self.indexqual.is_null() {
            if visitor.visit_index_scan_quals(unsafe { &*self.indexqual }).is_break() {
                return Traversal::Break;
            }
        }
        if !self.indexorderby.is_null() {
            if visitor.visit_index_scan_orderby(unsafe { &*self.indexorderby }).is_break() {
                return Traversal::Break;
            }
        }
        Traversal::Continue
    }
}

impl PgNodeWalk for pg_sys::IndexOnlyScan {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> Traversal {
        if !self.indexqual.is_null() {
            if visitor.visit_index_scan_quals(unsafe { &*self.indexqual }).is_break() {
                return Traversal::Break;
            }
        }
        if !self.indexorderby.is_null() {
            if visitor.visit_index_scan_orderby(unsafe { &*self.indexorderby }).is_break() {
                return Traversal::Break;
            }
        }
        Traversal::Continue
    }
}

impl PgNodeWalk for pg_sys::BitmapIndexScan {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> Traversal {
        if !self.indexqual.is_null() {
            if visitor.visit_index_scan_quals(unsafe { &*self.indexqual }).is_break() {
                return Traversal::Break;
            }
        }
        Traversal::Continue
    }
}

impl_pg_node_walk!(BitmapHeapScan);

impl PgNodeWalk for pg_sys::TidScan {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> Traversal {
        if !self.tidquals.is_null() {
            if visitor.visit_list(unsafe { &*self.tidquals }).is_break() {
                return Traversal::Break;
            }
        }
        Traversal::Continue
    }
}

impl PgNodeWalk for pg_sys::TidRangeScan {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> Traversal {
        if !self.tidrangequals.is_null() {
            if visitor.visit_list(unsafe { &*self.tidrangequals }).is_break() {
                return Traversal::Break;
            }
        }
        Traversal::Continue
    }
}

impl PgNodeWalk for pg_sys::Join {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> Traversal {
        if !self.joinqual.is_null() {
            if visitor.visit_join_qual(unsafe { &*self.joinqual }).is_break() {
                return Traversal::Break;
            }
        }
        Traversal::Continue
    }
}

impl_pg_node_walk!(NestLoop);

impl PgNodeWalk for pg_sys::MergeJoin {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> Traversal {
        if !self.mergeclauses.is_null() {
            if visitor.visit_merge_join_clauses(unsafe { &*self.mergeclauses }).is_break() {
                return Traversal::Break;
            }
        }
        Traversal::Continue
    }
}

impl PgNodeWalk for pg_sys::HashJoin {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> Traversal {
        if !self.hashclauses.is_null() {
            if visitor.visit_hash_join_clauses(unsafe { &*self.hashclauses }).is_break() {
                return Traversal::Break;
            }
        }
        Traversal::Continue
    }
}
impl_pg_node_walk!(Material);

impl PgNodeWalk for pg_sys::Limit {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> Traversal {
        if !self.limitOffset.is_null() {
            if unsafe { &*self.limitOffset }.walk(visitor).is_break() {
                return Traversal::Break;
            }
        }
        if !self.limitCount.is_null() {
            if unsafe { &*self.limitCount }.walk(visitor).is_break() {
                return Traversal::Break;
            }
        }
        Traversal::Continue
    }
}

impl PgNodeWalk for pg_sys::ModifyTable {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> Traversal {
        if !self.resultRelations.is_null() {
            if visitor.visit_modify_table_result_relations(unsafe { &*self.resultRelations }).is_break() {
                return Traversal::Break;
            }
        }
        Traversal::Continue
    }
}

impl PgNodeWalk for pg_sys::Agg {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> Traversal {
        if !self.groupingSets.is_null() {
            if visitor.visit_list(unsafe { &*self.groupingSets }).is_break() {
                return Traversal::Break;
            }
        }
        if !self.chain.is_null() {
            if visitor.visit_list(unsafe { &*self.chain }).is_break() {
                return Traversal::Break;
            }
        }
        Traversal::Continue
    }
}

impl_pg_node_walk!(Sort);

impl PgNodeWalk for pg_sys::Append {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> Traversal {
        if !self.appendplans.is_null() {
            if visitor.visit_append_plans(unsafe { &*self.appendplans }).is_break() {
                return Traversal::Break;
            }
        }
        Traversal::Continue
    }
}

impl PgNodeWalk for pg_sys::MergeAppend {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> Traversal {
        if !self.mergeplans.is_null() {
            if visitor.visit_merge_append_plans(unsafe { &*self.mergeplans }).is_break() {
                return Traversal::Break;
            }
        }
        Traversal::Continue
    }
}

impl PgNodeWalk for pg_sys::BitmapAnd {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> Traversal {
        if !self.bitmapplans.is_null() {
            if visitor.visit_bitmap_plans(unsafe { &*self.bitmapplans }).is_break() {
                return Traversal::Break;
            }
        }
        Traversal::Continue
    }
}

impl PgNodeWalk for pg_sys::BitmapOr {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> Traversal {
        if !self.bitmapplans.is_null() {
            if visitor.visit_bitmap_plans(unsafe { &*self.bitmapplans }).is_break() {
                return Traversal::Break;
            }
        }
        Traversal::Continue
    }
}

impl_pg_node_walk!(RecursiveUnion);

impl PgNodeWalk for pg_sys::ValuesScan {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> Traversal {
        if !self.values_lists.is_null() {
            if visitor.visit_values_scan_lists(unsafe { &*self.values_lists }).is_break() {
                return Traversal::Break;
            }
        }
        Traversal::Continue
    }
}

impl_pg_node_walk!(CteScan);
impl_pg_node_walk!(Gather);
impl_pg_node_walk!(GatherMerge);

impl PgNodeWalk for pg_sys::Hash {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> Traversal {
        if !self.hashkeys.is_null() {
            if visitor.visit_hash_keys(unsafe { &*self.hashkeys }).is_break() {
                return Traversal::Break;
            }
        }
        Traversal::Continue
    }
}

impl PgNodeWalk for pg_sys::LockRows {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> Traversal {
        if !self.rowMarks.is_null() {
            if visitor.visit_lock_rows_marks(unsafe { &*self.rowMarks }).is_break() {
                return Traversal::Break;
            }
        }
        Traversal::Continue
    }
}

impl_pg_node_walk!(WindowAgg);
impl_pg_node_walk!(Unique);
impl_pg_node_walk!(SetOp);

impl PgNodeWalk for pg_sys::TargetEntry {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> Traversal {
        if !self.expr.is_null() {
            if unsafe { &*self.expr }.as_node().walk(visitor).is_break() {
                return Traversal::Break;
            }
        }
        Traversal::Continue
    }
}

impl PgNodeWalk for pg_sys::PartitionPruneInfo {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> Traversal {
        if !self.prune_infos.is_null() {
            if visitor.visit_partition_prune_infos(unsafe { &*self.prune_infos }).is_break() {
                return Traversal::Break;
            }
        }
        Traversal::Continue
    }
}

impl PgNodeWalk for pg_sys::PartitionedRelPruneInfo {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> Traversal {
        if !self.initial_pruning_steps.is_null() {
            if visitor.visit_pruning_steps(unsafe { &*self.initial_pruning_steps }).is_break() {
                return Traversal::Break;
            }
        }
        if !self.exec_pruning_steps.is_null() {
            if visitor.visit_pruning_steps(unsafe { &*self.exec_pruning_steps }).is_break() {
                return Traversal::Break;
            }
        }
        Traversal::Continue
    }
}

impl PgNodeWalk for pg_sys::PartitionPruneStepOp {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> Traversal {
        if !self.exprs.is_null() {
            if visitor.visit_list(unsafe { &*self.exprs }).is_break() {
                return Traversal::Break;
            }
        }
        Traversal::Continue
    }
}

impl_pg_node_walk!(PartitionPruneStepCombine);
impl_pg_node_walk!(PlanRowMark);

impl_pg_node_walk!(Var);
impl_pg_node_walk!(Const);

impl PgNodeWalk for pg_sys::OpExpr {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> Traversal {
        if !self.args.is_null() {
            if visitor.visit_list(unsafe { &*self.args }).is_break() {
                return Traversal::Break;
            }
        }
        Traversal::Continue
    }
}

impl PgNodeWalk for pg_sys::FuncExpr {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> Traversal {
        if !self.args.is_null() {
            if visitor.visit_list(unsafe { &*self.args }).is_break() {
                return Traversal::Break;
            }
        }
        Traversal::Continue
    }
}

impl PgNodeWalk for pg_sys::BoolExpr {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> Traversal {
        if !self.args.is_null() {
            if visitor.visit_list(unsafe { &*self.args }).is_break() {
                return Traversal::Break;
            }
        }
        Traversal::Continue
    }
}

impl PgNodeWalk for pg_sys::SubLink {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> Traversal {
        if !self.testexpr.is_null() {
            if unsafe { &*self.testexpr }.walk(visitor).is_break() {
                return Traversal::Break;
            }
        }
        if !self.subselect.is_null() {
            if unsafe { &*self.subselect }.walk(visitor).is_break() {
                return Traversal::Break;
            }
        }
        Traversal::Continue
    }
}

impl PgNodeWalk for pg_sys::SubPlan {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> Traversal {
        if !self.testexpr.is_null() {
            if unsafe { &*self.testexpr }.walk(visitor).is_break() {
                return Traversal::Break;
            }
        }
        if !self.args.is_null() {
            if visitor.visit_list(unsafe { &*self.args }).is_break() {
                return Traversal::Break;
            }
        }
        Traversal::Continue
    }
}

impl PgNodeWalk for pg_sys::ScalarArrayOpExpr {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> Traversal {
        if !self.args.is_null() {
            if visitor.visit_list(unsafe { &*self.args }).is_break() {
                return Traversal::Break;
            }
        }
        Traversal::Continue
    }
}
impl_pg_node_walk!(RTEPermissionInfo);

impl PgNodeWalk for pg_sys::AppendRelInfo {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> Traversal {
        if !self.translated_vars.is_null() {
            if visitor.visit_list(unsafe { &*self.translated_vars }).is_break() {
                return Traversal::Break;
            }
        }
        Traversal::Continue
    }
}

impl_pg_node_walk!(PlanInvalItem);

impl PgNodeWalk for pg_sys::PlannedStmt {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> Traversal {
        if self.commandType == pg_sys::CmdType::CMD_UTILITY && !self.utilityStmt.is_null() {
            return unsafe { &*self.utilityStmt }.walk(visitor);
        }
        if !self.planTree.is_null() {
            if visitor.visit_planned_stmt_plan_tree(unsafe { &*self.planTree }).is_break() {
                return Traversal::Break;
            }
        }
        if !self.rtable.is_null() {
            if visitor.visit_planned_stmt_rtable(unsafe { &*self.rtable }).is_break() {
                return Traversal::Break;
            }
        }
        if !self.subplans.is_null() {
            if visitor.visit_planned_stmt_subplans(unsafe { &*self.subplans }).is_break() {
                return Traversal::Break;
            }
        }
        if !self.permInfos.is_null() {
            if visitor.visit_planned_stmt_perm_infos(unsafe { &*self.permInfos }).is_break() {
                return Traversal::Break;
            }
        }
        if !self.rowMarks.is_null() {
            if visitor.visit_planned_stmt_row_marks(unsafe { &*self.rowMarks }).is_break() {
                return Traversal::Break;
            }
        }
        if !self.invalItems.is_null() {
            if visitor.visit_planned_stmt_inval_items(unsafe { &*self.invalItems }).is_break() {
                return Traversal::Break;
            }
        }
        if !self.appendRelations.is_null() {
            if visitor.visit_planned_stmt_append_relations(unsafe { &*self.appendRelations }).is_break() {
                return Traversal::Break;
            }
        }
        if !self.partPruneInfos.is_null() {
            if visitor.visit_planned_stmt_part_prune_infos(unsafe { &*self.partPruneInfos }).is_break() {
                return Traversal::Break;
            }
        }
        Traversal::Continue
    }
}
