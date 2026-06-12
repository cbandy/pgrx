# Traversing `PlannedStmt` for DDL, Utility, and DML Statements in `pgrx`

In PostgreSQL, the planner produces a `PlannedStmt` for all statements.
- **Utility/DDL commands** (like `CREATE TABLE`, `ALTER TABLE`, etc.) are wrapped inside a `PlannedStmt` with `commandType == CmdType::CMD_UTILITY`. The raw parse tree is preserved in the `utilityStmt` field.
- **DML statements** (like `SELECT`, `UPDATE`, `INSERT`, `DELETE`, and `MERGE`) contain structured query plan trees in the `planTree` field, along with auxiliary information like the `rtable` (range table) and `subplans`.

As noted in `postgres/src/backend/nodes/README`, output serialization (such as `nodeToString` or `outNode`) for utility statements and raw parse trees is incomplete and largely unsupported. Therefore, analyzing or inspecting planned statements requires programmatically traversing the in-memory node trees.

This document details the design of a type-safe casting and visitor/walker traversal system in Rust to analyze `PlannedStmt` objects in `pgrx`.

---

## 1. Traversal Control Enum

Using booleans for traversal control (e.g., returning `true` or `false`) is confusing and error-prone. We define a dedicated `TraversalControl` enum:

```rust
/// Controls the traversal flow of the AST walker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraversalControl {
    /// Continue walking the AST tree.
    Continue,
    /// Stop/short-circuit the AST walk early.
    Break,
}

impl TraversalControl {
    /// Returns `true` if the traversal should stop.
    #[inline]
    pub fn is_break(&self) -> bool {
        matches!(self, TraversalControl::Break)
    }
}
```

---

## 2. The Casting System (Inspired by PR #1048)

### 2.1. Multiple NodeTags for a Single Struct

`HasNodeTag` defines a slice of tags to handle structs mapped to multiple `NodeTag` values (e.g. `pg_sys::List` covers `T_List`, `T_IntList`, `T_OidList`, and `T_XidList`):

```rust
pub trait HasNodeTag {
    const NODE_TAGS: &'static [pg_sys::NodeTag];
}

macro_rules! impl_has_node_tag {
    ($struct_name:ident, [$($tag_variant:ident),+]) => {
        impl HasNodeTag for pg_sys::$struct_name {
            const NODE_TAGS: &'static [pg_sys::NodeTag] = &[
                $(pg_sys::NodeTag::$tag_variant),+
            ];
        }
    };
    ($struct_name:ident, $tag_variant:ident) => {
        impl HasNodeTag for pg_sys::$struct_name {
            const NODE_TAGS: &'static [pg_sys::NodeTag] = &[pg_sys::NodeTag::$tag_variant];
        }
    };
}

impl_has_node_tag!(CreateStmt, T_CreateStmt);
impl_has_node_tag!(AlterTableStmt, T_AlterTableStmt);
impl_has_node_tag!(AlterTableCmd, T_AlterTableCmd);
impl_has_node_tag!(DropStmt, T_DropStmt);
impl_has_node_tag!(TruncateStmt, T_TruncateStmt);
impl_has_node_tag!(IndexStmt, T_IndexStmt);
impl_has_node_tag!(RangeVar, T_RangeVar);
impl_has_node_tag!(List, [T_List, T_IntList, T_OidList, T_XidList]);
impl_has_node_tag!(RangeTblEntry, T_RangeTblEntry);
impl_has_node_tag!(SeqScan, T_SeqScan);
impl_has_node_tag!(ModifyTable, T_ModifyTable);
impl_has_node_tag!(Agg, T_Agg);
impl_has_node_tag!(Sort, T_Sort);
impl_has_node_tag!(TargetEntry, T_TargetEntry);
```

### 2.2. Downcasting Trait

`PgNodeTryCast` performs fallible downcasting. `Option<Self>` is used (not `Result`) because a tag mismatch is a simple structural check with no meaningful error payload, and matches `std::any::Any::downcast_ref` conventions.

```rust
pub trait PgNodeTryCast<T> {
    fn try_cast_from(node: T) -> Option<Self>
    where
        Self: Sized;
}

impl<'a, B: HasNodeTag> PgNodeTryCast<&'a pg_sys::Node> for &'a B {
    fn try_cast_from(node: &'a pg_sys::Node) -> Option<Self> {
        if B::NODE_TAGS.contains(&node.type_) {
            // SAFETY: The type tag matched, verifying the underlying layout.
            unsafe { Some(&*(node as *const pg_sys::Node as *const B)) }
        } else {
            None
        }
    }
}

impl<'a, B: HasNodeTag> PgNodeTryCast<&'a mut pg_sys::Node> for &'a mut B {
    fn try_cast_from(node: &'a mut pg_sys::Node) -> Option<Self> {
        if B::NODE_TAGS.contains(&node.type_) {
            // SAFETY: The type tag matched, verifying the underlying layout.
            unsafe { Some(&mut *(node as *mut pg_sys::Node as *mut B)) }
        } else {
            None
        }
    }
}
```

---

## 3. Traversal and Visitor Traits

Following the design principles of `nodeFuncs.c` (`expression_tree_walker`):
1. `PgNodeWalk`: implemented by each struct to specify how to recurse into its child fields.
2. `PlannedStmtVisitor`: implemented by users; each visitor hook receives a **concrete, field-specific type** so that the visitor always knows exactly which part of the AST it is looking at.

> [!IMPORTANT]
> **Field-specific hooks, not generic `visit_list`.**
> Because multiple `*mut List` fields in a struct like `IndexStmt` (e.g. `indexParams` vs `indexIncludingParams`) are semantically distinct, routing them all through a single `visit_list` hook would make it impossible for a visitor to know which field it received. Each pointer field therefore gets its own named hook. The generic `visit_list` hook is retained **only** as a catch-all inside `Node::walk` for unrecognised list nodes encountered at a raw `Node` pointer level.

```rust
pub trait PgNodeWalk {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> TraversalControl;
}

pub trait PlannedStmtVisitor {
    // ---- Generic catch-all -------------------------------------------------
    /// Called for any node type not matched by a more specific hook.
    /// Defaults to `Continue` (no-op).
    fn visit_node(&mut self, _node: &pg_sys::Node) -> TraversalControl {
        TraversalControl::Continue
    }

    /// Generic list catch-all — called from `Node::walk` when the list type
    /// is not otherwise dispatched to a concrete field hook.
    fn visit_list(&mut self, list: &pg_sys::List) -> TraversalControl {
        let node = unsafe { &*(list as *const pg_sys::List as *const pg_sys::Node) };
        if self.visit_node(node).is_break() {
            return TraversalControl::Break;
        }
        list.walk(self)
    }

    // ---- Statement-level / DML hooks ---------------------------------------------
    fn visit_create_stmt(&mut self, stmt: &pg_sys::CreateStmt) -> TraversalControl { ... }
    fn visit_alter_table_stmt(&mut self, stmt: &pg_sys::AlterTableStmt) -> TraversalControl { ... }
    fn visit_alter_table_cmd(&mut self, cmd: &pg_sys::AlterTableCmd) -> TraversalControl { ... }
    fn visit_drop_stmt(&mut self, stmt: &pg_sys::DropStmt) -> TraversalControl { ... }
    fn visit_truncate_stmt(&mut self, stmt: &pg_sys::TruncateStmt) -> TraversalControl { ... }
    fn visit_index_stmt(&mut self, stmt: &pg_sys::IndexStmt) -> TraversalControl { ... }

    // ---- DML / Plan hooks -----------------------------------------------
    fn visit_seq_scan(&mut self, scan: &pg_sys::SeqScan) -> TraversalControl {
        if self.visit_plan(unsafe { &scan.scan.plan }).is_break() { return TraversalControl::Break; }
        scan.walk(self)
    }
    fn visit_agg(&mut self, agg: &pg_sys::Agg) -> TraversalControl {
        if self.visit_plan(&agg.plan).is_break() { return TraversalControl::Break; }
        agg.walk(self)
    }
    // ...

    // ---- Leaf / shared hooks -----------------------------------------------
    fn visit_range_var(&mut self, range_var: &pg_sys::RangeVar) -> TraversalControl { ... }
    fn visit_range_tbl_entry(&mut self, rte: &pg_sys::RangeTblEntry) -> TraversalControl { ... }
    fn visit_target_entry(&mut self, te: &pg_sys::TargetEntry) -> TraversalControl { ... }

    // ---- CreateStmt field hooks --------------------------------------------
    fn visit_create_table_elts(&mut self, list: &pg_sys::List) -> TraversalControl { self.visit_list(list) }
    // ...

    // ---- PlannedStmt field hooks ------------------------------------------
    fn visit_planned_stmt_rtable(&mut self, list: &pg_sys::List) -> TraversalControl { self.visit_list(list) }
    fn visit_planned_stmt_subplans(&mut self, list: &pg_sys::List) -> TraversalControl { self.visit_list(list) }
    fn visit_planned_stmt_plan_tree(&mut self, plan: &pg_sys::Plan) -> TraversalControl {
        unsafe { &*(plan as *const pg_sys::Plan as *const pg_sys::Node) }.walk(self)
    }
}
```

---

## 4. Walk Implementations

### 4.1. `pg_sys::Node` and `pg_sys::PlannedStmt`

```rust
impl PgNodeWalk for pg_sys::Node {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> TraversalControl {
        match self.type_ {
            pg_sys::NodeTag::T_CreateStmt => {
                let stmt = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::CreateStmt) };
                visitor.visit_create_stmt(stmt)
            }
            // ... routing for all supported NodeTags ...
            pg_sys::NodeTag::T_List => {
                let list = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::List) };
                visitor.visit_list(list)
            }
            _ => visitor.visit_node(self),
        }
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
```

### 4.2. Lists: Using `pgrx::list::*` Wrappers

```rust
impl PgNodeWalk for pg_sys::List {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> TraversalControl {
        // ... iterate over list and call node.walk(visitor) for each element ...
    }
}
```

### 4.3. DDL and DML Node Structs

Each `*mut RangeVar`, `*mut List`, or `*mut Plan` field routes through its own named visitor hook or recurses via `walk`, so the visitor always knows exactly which field it is examining:

```rust
// CREATE TABLE
impl PgNodeWalk for pg_sys::CreateStmt { ... }

// PLAN (Base for DML)
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
            if unsafe { &*(self.lefttree as *const pg_sys::Node) }.walk(visitor).is_break() {
                return TraversalControl::Break;
            }
        }
        // ...
        TraversalControl::Continue
    }
}

// SEQ SCAN
impl PgNodeWalk for pg_sys::SeqScan {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> TraversalControl {
        self.scan.plan.walk(visitor)
    }
}
```

---

## 5. Theoretical Evaluation of DML Support

### 5.1. Appropriateness
The Visitor pattern is highly appropriate for DML. While DDL statements are "shallow" parse trees, DML plans are "deep" trees of `Plan` nodes. Programmatic traversal is the standard way in PostgreSQL (e.g., `planstate_tree_walker`) to analyze these structures.

### 5.2. Depth of Traversal
- **Plan Node Traversal:** Essential for understanding the query strategy (e.g., identifying SeqScans or Joins).
- **Expression Traversal:** Granularly identifies columns and functions.
The system prioritizes Plan Node Traversal, but allows optional descent into expressions (via `visit_target_entry` and `visit_plan_qual`).

### 5.3. Range Table and Subplans
The Range Table (`rtable`) is the source of truth for relation access, and `subplans` contain execution trees for non-flattened subqueries. A complete analysis of a DML statement **must** include these, which the `PlannedStmt::walk` implementation ensures.

---

## 6. Concrete Example: Table Access Auditor

A visitor that identifies all tables accessed by *any* statement (DDL or DML):

```rust
pub struct TableAuditor {
    pub tables: HashSet<String>,
}

impl PlannedStmtVisitor for TableAuditor {
    // For DDL
    fn visit_range_var(&mut self, rv: &pg_sys::RangeVar) -> TraversalControl {
        self.add_relname(rv.relname);
        TraversalControl::Continue
    }

    // For DML
    fn visit_range_tbl_entry(&mut self, rte: &pg_sys::RangeTblEntry) -> TraversalControl {
        self.add_relid(rte.relid); // Resolve OID to name
        rte.walk(self) // Recurse into subqueries if any
    }
}
```
