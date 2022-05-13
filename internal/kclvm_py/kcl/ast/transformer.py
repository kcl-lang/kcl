# Copyright 2020 The KCL Authors. All rights reserved.

import kclvm.kcl.ast as ast


class TreeTransformer(ast.TreeWalker):
    """The TreeTransformer subclass that walks the abstract syntax tree and
    allows modification of nodes.

    The `TreeTransformer` will walk the AST and use the return value of the
    walker methods to replace or remove the old node.  If the return value of
    the walker method is ``None``, the node will be removed from its location,
    otherwise it is replaced with the return value.  The return value may be the
    original node in which case no replacement takes place.

    Keep in mind that if the node you're operating on has child nodes you must
    either transform the child nodes yourself or call the :meth:`generic_walk`
    method for the node first.

    For nodes that were part of a collection of statements (that applies to all
    statement nodes), the walker may also return a list of nodes rather than
    just a single node, e.g., `module` AST contains the `body` attribute, which
    is a list of statement

    Example:

        class ChangeSchemaExprNameTransformer(TreeTransformer):
            def walk_SchemaExpr(self, t: ast.SchemaExpr):
                if t.name.get_name() == 'Person':
                    t.name.set_name('PersonNew')
                return t

    Usually we can use the transformer like this::

       node = YourTransformer().walk(node)

    """

    # Base transformer functions

    def get_node_name(self, t: ast.AST):
        """Get the ast.AST node name"""
        assert isinstance(t, ast.AST)
        return t.type

    def generic_walk(self, t: ast.AST):
        """Called if no explicit walker function exists for a node."""
        for field, old_value in ast.iter_fields(t):
            if isinstance(old_value, list):
                new_values = []
                for value in old_value:
                    if isinstance(value, ast.AST):
                        value = self.walk(value)
                        if value is None:
                            continue
                        elif not isinstance(value, ast.AST):
                            new_values.extend(value)
                            continue
                    elif isinstance(value, list):
                        value = [self.walk(v) for v in value]
                    new_values.append(value)
                old_value[:] = new_values
            elif isinstance(old_value, ast.AST):
                new_node = self.walk(old_value)
                if new_node is None:
                    delattr(t, field)
                else:
                    setattr(t, field, new_node)
        return t
