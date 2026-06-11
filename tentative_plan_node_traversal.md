# Traversing `PlannedStmt` for DDL and Utility Statements in `pgrx`

In PostgreSQL, the planner produces a `PlannedStmt` for all statements. While DML statements (like `SELECT`, `UPDATE`, `INSERT`, and `DELETE`) contain structured query plan trees, utility commands (which include DDL statements like `CREATE TABLE`, `ALTER TABLE`, `CREATE INDEX`, and `DROP`) are wrapped inside a `PlannedStmt` with the command type `CmdType::CMD_UTILITY`. In these cases, the raw parse tree is preserved in the `utilityStmt` field as a `pg_sys::Node` pointer.

As noted in `postgres/src/backend/nodes/README`, output serialization (such as `nodeToString` or `outNode`) for utility statements and raw parse trees is incomplete and largely unsupported. Therefore, analyzing or inspecting planned DDL statements requires programmatically traversing the in-memory node trees.

This document details the design of a type-safe casting and visitor/walker traversal system in Rust to analyze `PlannedStmt` objects representing DDL and utility statements in `pgrx`.

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

    // ---- Statement-level hooks ---------------------------------------------
    fn visit_create_stmt(&mut self, stmt: &pg_sys::CreateStmt) -> TraversalControl {
        let node = unsafe { &*(stmt as *const pg_sys::CreateStmt as *const pg_sys::Node) };
        if self.visit_node(node).is_break() { return TraversalControl::Break; }
        stmt.walk(self)
    }

    fn visit_alter_table_stmt(&mut self, stmt: &pg_sys::AlterTableStmt) -> TraversalControl {
        let node = unsafe { &*(stmt as *const pg_sys::AlterTableStmt as *const pg_sys::Node) };
        if self.visit_node(node).is_break() { return TraversalControl::Break; }
        stmt.walk(self)
    }

    fn visit_alter_table_cmd(&mut self, cmd: &pg_sys::AlterTableCmd) -> TraversalControl {
        let node = unsafe { &*(cmd as *const pg_sys::AlterTableCmd as *const pg_sys::Node) };
        if self.visit_node(node).is_break() { return TraversalControl::Break; }
        cmd.walk(self)
    }

    fn visit_drop_stmt(&mut self, stmt: &pg_sys::DropStmt) -> TraversalControl {
        let node = unsafe { &*(stmt as *const pg_sys::DropStmt as *const pg_sys::Node) };
        if self.visit_node(node).is_break() { return TraversalControl::Break; }
        stmt.walk(self)
    }

    fn visit_truncate_stmt(&mut self, stmt: &pg_sys::TruncateStmt) -> TraversalControl {
        let node = unsafe { &*(stmt as *const pg_sys::TruncateStmt as *const pg_sys::Node) };
        if self.visit_node(node).is_break() { return TraversalControl::Break; }
        stmt.walk(self)
    }

    fn visit_index_stmt(&mut self, stmt: &pg_sys::IndexStmt) -> TraversalControl {
        let node = unsafe { &*(stmt as *const pg_sys::IndexStmt as *const pg_sys::Node) };
        if self.visit_node(node).is_break() { return TraversalControl::Break; }
        stmt.walk(self)
    }

    // ---- Leaf / shared hooks -----------------------------------------------
    fn visit_range_var(&mut self, range_var: &pg_sys::RangeVar) -> TraversalControl {
        let node = unsafe { &*(range_var as *const pg_sys::RangeVar as *const pg_sys::Node) };
        if self.visit_node(node).is_break() { return TraversalControl::Break; }
        range_var.walk(self)
    }

    // ---- CreateStmt field hooks --------------------------------------------
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

    // ---- AlterTableStmt field hooks ----------------------------------------
    fn visit_alter_table_cmds(&mut self, list: &pg_sys::List) -> TraversalControl {
        self.visit_list(list)
    }

    // ---- DropStmt field hooks -----------------------------------------------
    fn visit_drop_objects(&mut self, list: &pg_sys::List) -> TraversalControl {
        self.visit_list(list)
    }

    // ---- TruncateStmt field hooks ------------------------------------------
    fn visit_truncate_relations(&mut self, list: &pg_sys::List) -> TraversalControl {
        self.visit_list(list)
    }

    // ---- IndexStmt field hooks ---------------------------------------------
    fn visit_index_params(&mut self, list: &pg_sys::List) -> TraversalControl {
        self.visit_list(list)
    }
    fn visit_index_including_params(&mut self, list: &pg_sys::List) -> TraversalControl {
        self.visit_list(list)
    }
    fn visit_index_options(&mut self, list: &pg_sys::List) -> TraversalControl {
        self.visit_list(list)
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
            // Generic list catch-all — no concrete field context available here.
            pg_sys::NodeTag::T_List => {
                let list = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::List) };
                visitor.visit_list(list)
            }
            _ => TraversalControl::Continue,
        }
    }
}

impl PgNodeWalk for pg_sys::PlannedStmt {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> TraversalControl {
        if self.commandType == pg_sys::CmdType::CMD_UTILITY && !self.utilityStmt.is_null() {
            let stmt = unsafe { &*self.utilityStmt };
            visitor.visit_node(stmt)
        } else {
            TraversalControl::Continue
        }
    }
}
```

### 4.2. Lists: Using `pgrx::list::*` Wrappers

```rust
impl PgNodeWalk for pg_sys::List {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> TraversalControl {
        let list_ptr = self as *const pg_sys::List as *mut pg_sys::List;
        crate::memcx::current_context(|mcx| unsafe {
            if let Some(list) = crate::list::List::<*mut core::ffi::c_void>::downcast_ptr_in_memcx(list_ptr, mcx) {
                for cell_ptr in list.iter() {
                    if !cell_ptr.is_null() {
                        let node = &*(*cell_ptr as *const pg_sys::Node);
                        if visitor.visit_node(node).is_break() {
                            return TraversalControl::Break;
                        }
                    }
                }
            }
            TraversalControl::Continue
        })
    }
}
```

### 4.3. DDL Node Structs

Each `*mut RangeVar` and `*mut List` field routes through its own named visitor hook, so the visitor always knows exactly which field it is examining:

```rust
// CREATE TABLE
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

// ALTER TABLE
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

// ALTER TABLE COMMAND
impl PgNodeWalk for pg_sys::AlterTableCmd {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> TraversalControl {
        // `def` is a generic Node* — no more specific hook available.
        if !self.def.is_null() {
            if visitor.visit_node(unsafe { &*self.def }).is_break() {
                return TraversalControl::Break;
            }
        }
        TraversalControl::Continue
    }
}

// DROP
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

// TRUNCATE
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

// CREATE INDEX
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
        TraversalControl::Continue // leaf node
    }
}
```

---

## 5. Concrete Example: Table Name Extractor

```rust
use std::ffi::CStr;

pub struct TableNameExtractor {
    pub extracted_relations: Vec<String>,
}

impl PlannedStmtVisitor for TableNameExtractor {
    fn visit_range_var(&mut self, range_var: &pg_sys::RangeVar) -> TraversalControl {
        if !range_var.relname.is_null() {
            let relname = unsafe { CStr::from_ptr(range_var.relname) };
            if let Ok(name_str) = relname.to_str() {
                self.extracted_relations.push(name_str.to_string());
            }
        }
        TraversalControl::Continue
    }
}

pub fn extract_ddl_tables(planned_stmt: &pg_sys::PlannedStmt) -> Vec<String> {
    let mut extractor = TableNameExtractor { extracted_relations: Vec::new() };
    planned_stmt.walk(&mut extractor);
    extractor.extracted_relations
}
```

A visitor that only cares about `INCLUDE` columns (distinct from indexed columns) can now simply override `visit_index_including_params` without any ambiguity:

```rust
impl PlannedStmtVisitor for IncludeColumnCollector {
    fn visit_index_params(&mut self, _list: &pg_sys::List) -> TraversalControl {
        // Skip: not interested in the indexed columns.
        TraversalControl::Continue
    }

    fn visit_index_including_params(&mut self, list: &pg_sys::List) -> TraversalControl {
        // Only INCLUDE columns arrive here — no need to inspect the pointer.
        list.walk(self)  // continue into list elements
    }
}
```

---

## 6. Cross-Version Maintenance and Compatibility

Because fields inside structures change between major PostgreSQL versions, we propose two strategies:

### Strategy A: Version-Specific `cfg` Gates
```rust
impl PgNodeWalk for pg_sys::CreateStmt {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> TraversalControl {
        // ... common fields ...

        #[cfg(any(feature = "pg14", feature = "pg15", feature = "pg16", feature = "pg17", feature = "pg18"))]
        if !self.partbound.is_null() {
            if visitor.visit_node(unsafe { &*(self.partbound as *const _ as *const pg_sys::Node) }).is_break() {
                return TraversalControl::Break;
            }
        }

        TraversalControl::Continue
    }
}
```

### Strategy B: Automatic Code Generation via `pgrx-bindgen` (Recommended)
Extend `pgrx-bindgen/src/build.rs` to:
1. Detect all `PgNode` structs (DFS from `NodeTag` roots).
2. Inspect each field: if it is `*mut RangeVar` or `*mut List`, generate a named visitor hook and the corresponding walk call.
3. Emit the `PgNodeWalk` impls directly into the per-version generated module.

This guarantees generated walkers are always in sync with the active Postgres target.
