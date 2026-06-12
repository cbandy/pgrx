# Traversing `PlannedStmt` for DDL, Utility, and DML Statements in `pgrx`

In PostgreSQL, the planner produces a `PlannedStmt` for all statements.
- **Utility/DDL commands** (like `CREATE TABLE`, `ALTER TABLE`, etc.) are wrapped inside a `PlannedStmt` with `commandType == CmdType::CMD_UTILITY`. The raw parse tree is preserved in the `utilityStmt` field.
- **DML statements** (like `SELECT`, `UPDATE`, `INSERT`, `DELETE`, and `MERGE`) contain structured query plan trees in the `planTree` field, along with auxiliary information like the `rtable` (range table), `permInfos` (permissions), `rowMarks` (locking), and `subplans`.

As noted in `postgres/src/backend/nodes/README`, output serialization (such as `nodeToString` or `outNode`) for utility statements and raw parse trees is incomplete and largely unsupported. Therefore, analyzing or inspecting planned statements requires programmatically traversing the in-memory node trees.

This document details the design of a type-safe casting and visitor/walker traversal system in Rust to analyze `PlannedStmt` objects in `pgrx`.

---

## 1. Traversal Enum

We define a dedicated `Traversal` enum to control the flow of the AST walk:

```rust
/// Controls the traversal flow of the AST walker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Traversal {
    /// Continue walking the AST tree.
    Continue,
    /// Stop/short-circuit the AST walk early.
    Break,
}
```

---

## 2. The Casting and Upcasting System

### 2.1. Safe Upcasting with `AsPgNode`

To avoid manual and repetitive `unsafe` casts to `pg_sys::Node`, we provide the `AsPgNode` trait. This trait is automatically implemented for all supported nodes via the `has_node_tag!` and `as_pg_node!` macros.

```rust
pub trait AsPgNode {
    fn as_node(&self) -> &pg_sys::Node;
    fn as_node_mut(&mut self) -> &mut pg_sys::Node;
}
```

### 2.2. Multiple NodeTags and `HasNodeTag`

`HasNodeTag` defines a slice of tags to handle structs mapped to multiple `NodeTag` values (e.g. `pg_sys::List` covers `T_List`, `T_IntList`, `T_OidList`, and `T_XidList`). It requires `AsPgNode`.

```rust
pub trait HasNodeTag: AsPgNode {
    const NODE_TAGS: &'static [pg_sys::NodeTag];
}

macro_rules! has_node_tag {
    ($struct_name:ident, [$($tag_variant:ident),+]) => { ... };
    ($struct_name:ident, $tag_variant:ident) => { ... };
}

has_node_tag!(CreateStmt, T_CreateStmt);
has_node_tag!(RangeVar, T_RangeVar);
has_node_tag!(List, [T_List, T_IntList, T_OidList, T_XidList]);
// ... other DDL and DML nodes ...
```

---

## 3. Traversal and Visitor Traits

Following the design principles of `nodeFuncs.c` (`expression_tree_walker`):
1. `PgNodeWalk`: implemented by each struct to specify how to recurse into its child fields.
2. `PlannedStmtVisitor`: implemented by users; each visitor hook receives a **concrete, field-specific type**.

### 3.1. DML Support

For DML statements, the visitor provides hooks for `Plan` nodes and auxiliary structures:

- **`visit_planned_stmt_plan_tree`**: The entry point for the execution plan.
- **`visit_planned_stmt_rtable`**: Allows inspection of the Range Table (the relations involved).
- **`visit_planned_stmt_subplans`**: Allows inspection of subqueries.
- **Hierarchical Plan Hooks**: Hooks for plan types like `visit_seq_scan`, `visit_scan`, and `visit_plan`.

```rust
pub trait PlannedStmtVisitor {
    // ---- Generic catch-all -------------------------------------------------
    fn visit_node(&mut self, _node: &pg_sys::Node) -> Traversal {
        Traversal::Continue
    }

    // ---- DDL / Utility hooks ---------------------------------------------
    fn visit_create_stmt(&mut self, stmt: &pg_sys::CreateStmt) -> Traversal {
        if self.visit_node(stmt.as_node()).is_break() { return Traversal::Break; }
        stmt.walk(self)
    }
    // ...

    // ---- DML / Plan hooks -----------------------------------------------
    fn visit_plan(&mut self, plan: &pg_sys::Plan) -> Traversal {
        if self.visit_node(plan.as_node()).is_break() { return Traversal::Break; }
        plan.walk(self)
    }

    fn visit_scan(&mut self, scan: &pg_sys::Scan) -> Traversal {
        if self.visit_plan(&scan.plan).is_break() { return Traversal::Break; }
        Traversal::Continue
    }

    fn visit_seq_scan(&mut self, scan: &pg_sys::SeqScan) -> Traversal {
        if self.visit_scan(&scan.scan).is_break() { return Traversal::Break; }
        scan.walk(self)
    }
    // ...
}
```

---

## 4. Walk Implementations

### 4.1. `pg_sys::Node` and `pg_sys::PlannedStmt`

```rust
impl PgNodeWalk for pg_sys::Node {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> Traversal {
        match self.type_ {
            pg_sys::NodeTag::T_CreateStmt => {
                let stmt = unsafe { &*(self as *const pg_sys::Node as *const pg_sys::CreateStmt) };
                visitor.visit_create_stmt(stmt)
            }
            // ... routing for all supported NodeTags ...
            _ => visitor.visit_node(self),
        }
    }
}

impl PgNodeWalk for pg_sys::PlannedStmt {
    fn walk<V: PlannedStmtVisitor + ?Sized>(&self, visitor: &mut V) -> Traversal {
        if self.commandType == pg_sys::CmdType::CMD_UTILITY && !self.utilityStmt.is_null() {
            return unsafe { &*self.utilityStmt }.walk(visitor);
        }
        // ... walk planTree, rtable, subplans, permInfos, rowMarks, invalItems, etc. ...
        Traversal::Continue
    }
}
```

---

## 5. Theoretical Evaluation of DML Support

### 5.1. Appropriateness
The Visitor pattern is highly appropriate for DML. Programmatic traversal is the standard way in PostgreSQL to analyze deep trees of `Plan` nodes.

### 5.2. Depth of Traversal
The system prioritizes **Plan Node Traversal** (strategy level), but allows optional descent into expressions via `visit_target_entry` and `visit_plan_qual`.

---

## 6. Concrete Example: Table Access Auditor

A visitor that identifies all tables accessed by *any* statement (DDL or DML):

```rust
pub struct TableAuditor {
    pub tables: HashSet<String>,
}

impl PlannedStmtVisitor for TableAuditor {
    // For DDL
    fn visit_range_var(&mut self, rv: &pg_sys::RangeVar) -> Traversal {
        // Safe access to relname via pg_sys utility or CStr
        Traversal::Continue
    }

    // For DML
    fn visit_range_tbl_entry(&mut self, rte: &pg_sys::RangeTblEntry) -> Traversal {
        // Resolve OID to name and collect
        rte.walk(self) // Recurse into subqueries if any
    }
}
```
