//LICENSE Copyright 2026-2026 PgCentral Foundation, Inc. <contact@pgcentral.org>
//LICENSE
//LICENSE All rights reserved.
//LICENSE
//LICENSE Use of this source code is governed by the MIT license that can be found in the LICENSE file.

#[cfg(any(test, feature = "pg_test"))]
#[pgrx::pg_schema]
mod tests {
    #[allow(unused_imports)]
    use crate as pgrx_unit_tests;
    use pgrx::list::List;
    use pgrx::nodes::{
        PlanStateVisitor, TreeVisitor, Walk, WalkQueryTree, walk_expression_tree,
        walk_planstate_tree, walk_query_tree, walk_raw_expression_tree,
    };
    use pgrx::{memcx, pg_sys, pg_test};

    #[pg_test]
    fn test_walk_expression_tree() {
        memcx::current_context(|mcx| {
            let mut child1 = pg_sys::Var::default();
            child1.xpr.type_ = pg_sys::NodeTag::T_Var;

            let mut child2 = pg_sys::Var::default();
            child2.xpr.type_ = pg_sys::NodeTag::T_Var;

            let mut list: List<'_, pgrx::void_mut_ptr> = List::Nil;
            list.unstable_push_in_context(core::ptr::from_mut(&mut child1).cast(), mcx);
            list.unstable_push_in_context(core::ptr::from_mut(&mut child2).cast(), mcx);

            struct CountingVisitor {
                count: usize,
            }
            impl TreeVisitor for CountingVisitor {
                fn visit(&mut self, _node: &pg_sys::Node) -> Walk {
                    self.count += 1;
                    Walk::Continue
                }
            }

            let mut visitor = CountingVisitor { count: 0 };
            let list_node = unsafe { &*(list.as_ptr() as *const pg_sys::Node) };
            walk_expression_tree(list_node, &mut visitor);

            assert_eq!(visitor.count, 2);
        });
    }

    #[pg_test]
    fn test_walk_raw_expression_tree() {
        memcx::current_context(|mcx| {
            let mut child1 = pg_sys::Var::default();
            child1.xpr.type_ = pg_sys::NodeTag::T_Var;

            let mut child2 = pg_sys::Var::default();
            child2.xpr.type_ = pg_sys::NodeTag::T_Var;

            let mut list: List<'_, pgrx::void_mut_ptr> = List::Nil;
            list.unstable_push_in_context(core::ptr::from_mut(&mut child1).cast(), mcx);
            list.unstable_push_in_context(core::ptr::from_mut(&mut child2).cast(), mcx);

            struct CountingVisitor {
                count: usize,
            }
            impl TreeVisitor for CountingVisitor {
                fn visit(&mut self, _node: &pg_sys::Node) -> Walk {
                    self.count += 1;
                    Walk::Continue
                }
            }

            let mut visitor = CountingVisitor { count: 0 };
            let list_node = unsafe { &*(list.as_ptr() as *const pg_sys::Node) };
            walk_raw_expression_tree(list_node, &mut visitor);

            assert_eq!(visitor.count, 2);
        });
    }

    #[pg_test]
    fn test_walk_query_tree() {
        memcx::current_context(|mcx| {
            let mut query = pg_sys::Query::default();
            query.type_ = pg_sys::NodeTag::T_Query;

            let mut expr = pg_sys::Var::default();
            expr.xpr.type_ = pg_sys::NodeTag::T_Var;

            let mut list: List<'_, pgrx::void_mut_ptr> = List::Nil;
            list.unstable_push_in_context(core::ptr::from_mut(&mut expr).cast(), mcx);

            query.targetList = list.into_ptr() as *mut pg_sys::List;

            struct CountingVisitor {
                count: usize,
            }
            impl TreeVisitor for CountingVisitor {
                fn visit(&mut self, _node: &pg_sys::Node) -> Walk {
                    self.count += 1;
                    Walk::Continue
                }
            }

            let mut visitor = CountingVisitor { count: 0 };
            walk_query_tree(&query, &mut visitor, WalkQueryTree::default());

            // It should visit targetList elements (at least 1)
            assert!(visitor.count > 0);
        });
    }

    #[pg_test]
    fn test_walk_planstate_tree() {
        let mut child1_plan = pg_sys::Plan::default();
        let mut child2_plan = pg_sys::Plan::default();
        let mut parent_plan = pg_sys::Plan::default();

        let mut child1 = pg_sys::PlanState::default();
        child1.type_ = pg_sys::NodeTag::T_ProjectSetState;
        child1.plan = &mut child1_plan;

        let mut child2 = pg_sys::PlanState::default();
        child2.type_ = pg_sys::NodeTag::T_ProjectSetState;
        child2.plan = &mut child2_plan;

        let mut parent = pg_sys::PlanState::default();
        parent.type_ = pg_sys::NodeTag::T_ResultState;
        parent.lefttree = &mut child1;
        parent.righttree = &mut child2;
        parent.plan = &mut parent_plan;

        struct PlanStateCountingVisitor {
            count: usize,
        }
        impl PlanStateVisitor for PlanStateCountingVisitor {
            fn visit(&mut self, _planstate: &pg_sys::PlanState) -> Walk {
                self.count += 1;
                Walk::Continue
            }
        }

        let mut visitor = PlanStateCountingVisitor { count: 0 };
        walk_planstate_tree(&parent, &mut visitor);

        assert_eq!(visitor.count, 2);
    }

    #[pg_test]
    fn test_walk_expression_tree_early_stop() {
        memcx::current_context(|mcx| {
            let mut child1 = pg_sys::Var::default();
            child1.xpr.type_ = pg_sys::NodeTag::T_Var;

            let mut child2 = pg_sys::Var::default();
            child2.xpr.type_ = pg_sys::NodeTag::T_Var;

            let mut list: List<'_, pgrx::void_mut_ptr> = List::Nil;
            list.unstable_push_in_context(core::ptr::from_mut(&mut child1).cast(), mcx);
            list.unstable_push_in_context(core::ptr::from_mut(&mut child2).cast(), mcx);

            struct StoppingVisitor {
                count: usize,
            }
            impl TreeVisitor for StoppingVisitor {
                fn visit(&mut self, _node: &pg_sys::Node) -> Walk {
                    self.count += 1;
                    Walk::Stop
                }
            }

            let mut visitor = StoppingVisitor { count: 0 };
            let list_node = unsafe { &*(list.as_ptr() as *const pg_sys::Node) };
            let result = walk_expression_tree(list_node, &mut visitor);

            assert_eq!(visitor.count, 1);
            assert!(matches!(result, Walk::Stop));
        });
    }

    #[pg_test]
    fn test_walk_expression_tree_null_handling() {
        memcx::current_context(|mcx| {
            let mut child = pg_sys::Var::default();
            child.xpr.type_ = pg_sys::NodeTag::T_Var;

            let mut list: List<'_, pgrx::void_mut_ptr> = List::Nil;
            list.unstable_push_in_context(core::ptr::null_mut(), mcx);
            list.unstable_push_in_context(core::ptr::from_mut(&mut child).cast(), mcx);

            struct CountingVisitor {
                count: usize,
            }
            impl TreeVisitor for CountingVisitor {
                fn visit(&mut self, _node: &pg_sys::Node) -> Walk {
                    self.count += 1;
                    Walk::Continue
                }
            }

            let mut visitor = CountingVisitor { count: 0 };
            let list_node = unsafe { &*(list.as_ptr() as *const pg_sys::Node) };
            let result = walk_expression_tree(list_node, &mut visitor);

            assert_eq!(visitor.count, 1);
            assert!(matches!(result, Walk::Continue));
        });
    }
}
