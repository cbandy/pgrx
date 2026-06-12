# Traversing PostgreSQL Node Trees in `pgrx`

PostgreSQL uses a unified `Node` infrastructure for almost all query processing stages. Analyzing a query requires programmatically traversing these trees.

- **Raw Parse Trees**: Literal SQL structure (`parsenodes.h`).
- **Analyzed Query Trees (`Query`)**: Resolved parsetrees after semantic analysis.
- **Planned Statements (`PlannedStmt`)**: The chosen execution strategy, containing a `Plan` tree.
- **Execution Plans (`Plan`)**: Structured trees for executor strategy.
- **Expressions (`Expr`)**: Leaf nodes like `Var`, `Const`, or internal nodes like `OpExpr` used throughout all trees.

This document details the design of a type-safe casting and visitor/walker traversal system in Rust for `pgrx`.

---

## 1. Traversal Enum

We define a dedicated `Traversal` enum to control the flow of the AST walk:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Traversal {
    Continue,
    Break,
}
```

---

## 2. The Casting and Upcasting System

To avoid manual `unsafe` casts, we use the `AsPgNode` trait for safe upcasting to the base `pg_sys::Node` type.

```rust
pub trait AsPgNode {
    fn as_node(&self) -> &pg_sys::Node;
    fn as_node_mut(&mut self) -> &mut pg_sys::Node;
}

// Automatically implemented for supported structs via macros:
has_node_tag!(SeqScan, T_SeqScan);
has_node_tag!(Var, T_Var);
```

---

## 3. Hierarchical Visitor and Walker Traits

The system uses a two-part design:
1. **`PgNodeWalk`**: Specifies how a struct recurses into its child fields.
2. **`PlannedStmtVisitor`**: Provides hooks for interception.

### 3.1. Hierarchical Dispatch

Visitor hooks are organized hierarchically. Specialized hooks call their base hooks to allow interception at any level of abstraction. To avoid redundant traversal of base fields, recursion (`walk`) is only triggered in "leaf" hooks.

```rust
pub trait PlannedStmtVisitor {
    // Base catch-all
    fn visit_node(&mut self, _node: &pg_sys::Node) -> Traversal { Traversal::Continue }

    // Middle-man hook: Intercepts all plans
    fn visit_plan(&mut self, plan: &pg_sys::Plan) -> Traversal {
        self.visit_node(plan.as_node())
    }

    // Middle-man hook: Intercepts all scans
    fn visit_scan(&mut self, scan: &pg_sys::Scan) -> Traversal {
        self.visit_plan(&scan.plan)
    }

    // Leaf hook: Intercepts specific node and triggers recursion
    fn visit_seq_scan(&mut self, scan: &pg_sys::SeqScan) -> Traversal {
        if self.visit_scan(&scan.scan).is_break() { return Traversal::Break; }
        scan.walk(self)
    }
}
```

---

## 4. Coverage

The system handles both DDL (Utility) and DML statements by supporting:
- **`PlannedStmt`**: Strategic execution wrapper.
- **`Query`**: Analyzed query trees (rewriter level).
- **Plan Nodes**: Extensive coverage of scans (`Seq`, `Index`, `Bitmap`), joins (`NestLoop`, `Merge`, `Hash`), and auxiliary nodes (`Agg`, `Sort`, `ModifyTable`).
- **Expressions**: Deep descent into `WHERE` clauses and `TARGET` lists via `OpExpr`, `FuncExpr`, `ScalarArrayOpExpr`, `BoolExpr`, `Var`, and `Const`.

---

## 5. Concrete Example: Column Usage Auditor

A visitor that identifies every column (`Var`) referenced in a query:

```rust
pub struct ColumnAuditor {
    pub columns: HashSet<(i32, i16)>, // (varno, varattno)
}

impl PlannedStmtVisitor for ColumnAuditor {
    fn visit_var(&mut self, var: &pg_sys::Var) -> Traversal {
        self.columns.insert((var.varno, var.varattno));
        Traversal::Continue
    }
}
```
