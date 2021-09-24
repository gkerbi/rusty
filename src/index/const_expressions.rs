// Copyright (c) 2020 Ghaith Hachem and Mathias Rieder

use crate::ast::AstStatement;
use generational_arena::{Arena, Iter};

pub type ConstId = generational_arena::Index;

/// wrapper around ConstExpression stored in the arena
/// changing expr allows to change the referenced const-expression
/// without aquiring a new ID in the arena
struct ConstWrapper {
    /// the constant expression
    expr: ConstExpression,
    /// the name of the data_type that this should resolve to (is this really always known?)
    target_type_name: String,
}

impl ConstWrapper {
    pub fn get_statement(&self) -> &AstStatement {
        match &self.expr {
            ConstExpression::Unresolved(statement) => statement,
            ConstExpression::Resolved(statement) => statement,
            ConstExpression::Unresolvable { statement, .. } => statement,
        }
    }
}

/// constant expressions registered here are wrapped behind this enum to indicate
/// whether this expression was already (potentially) resolved or not, or if a
/// resolving failed.
pub enum ConstExpression {
    Unresolved(AstStatement),
    Resolved(AstStatement),
    Unresolvable {
        statement: AstStatement,
        reason: String,
    },
}

impl ConstExpression {
    pub fn get_statement(&self) -> &AstStatement {
        match &self {
            ConstExpression::Unresolved(statement) => statement,
            ConstExpression::Resolved(statement) => statement,
            ConstExpression::Unresolvable { statement, .. } => statement,
        }
    }
}

#[derive(Default)]
pub struct ConstExpressions {
    expressions: Arena<ConstWrapper>,
}

impl ConstExpressions {
    pub fn new() -> ConstExpressions {
        ConstExpressions {
            expressions: Arena::new(),
        }
    }

    pub fn add_expression(&mut self, statement: AstStatement, target_type_name: String) -> ConstId {
        self.expressions.insert(ConstWrapper {
            expr: ConstExpression::Unresolved(statement),
            target_type_name,
        })
    }

    pub fn find_expression(&self, id: &ConstId) -> Option<&AstStatement> {
        self.expressions.get(*id).map(|it| it.get_statement())
    }

    pub fn find_expression_target_type(&self, id: &ConstId) -> Option<&str> {
        self.expressions
            .get(*id)
            .map(|it| it.target_type_name.as_str())
    }

    /// similar to `find_expression` but it does not return the `AstStatement` directly.
    /// it returns a ConstExpression wrapper that indicates whether this expression
    /// was successfully resolved yet or not
    pub fn find_const_expression(&self, id: &ConstId) -> Option<&ConstExpression> {
        self.expressions.get(*id).map(|it| &it.expr)
    }

    pub fn remove(&mut self, id: &ConstId) -> Option<(AstStatement, String)> {
        self.expressions.remove(*id).map(|it| match it.expr {
            ConstExpression::Unresolved(s) => (s, it.target_type_name),
            ConstExpression::Resolved(s) => (s, it.target_type_name),
            ConstExpression::Unresolvable { statement: s, .. } => (s, it.target_type_name),
        })
    }

    pub fn mark_resolved(
        &mut self,
        id: &ConstId,
        new_statement: AstStatement,
    ) -> Result<(), String> {
        let wrapper = self
            .expressions
            .get_mut(*id)
            .ok_or_else(|| format!("Cannot find constant expression with id: {:?}", id))?;

        wrapper.expr = ConstExpression::Resolved(new_statement);
        Ok(())
    }

    pub fn mark_unresolvable(&mut self, id: &ConstId, reason: &str) -> Result<(), String> {
        let wrapper = self
            .expressions
            .get_mut(*id)
            .ok_or_else(|| format!("Cannot find constant expression with id: {:?}", id))?;

        wrapper.expr = ConstExpression::Unresolvable {
            statement: wrapper.get_statement().clone(),
            reason: reason.to_string(),
        };
        Ok(())
    }

    /// adds the given constant expression to the constants arena and returns the ID to reference it
    pub fn add_constant_expression(&mut self, expr: AstStatement, target_type: String) -> ConstId {
        self.add_expression(expr, target_type)
    }

    /// convinience-method to add the constant exression if there is some, otherwhise not
    /// use this only as a shortcut if you have an Option<AstStatement> - e.g. an optional initializer.
    /// otherwhise use `add_constant_expression`
    pub fn maybe_add_constant_expression(
        &mut self,
        expr: Option<AstStatement>,
        targe_type_name: &str,
    ) -> Option<ConstId> {
        expr.map(|it| self.add_constant_expression(it, targe_type_name.to_string()))
    }

    /// convinience-method to query for an optional constant expression.
    /// if the given `id` is `None`, this method returns `None`
    /// use this only as a shortcut if you have an Option<ConstId> - e.g. an optional initializer.
    /// otherwhise use `get_constant_expression`
    pub fn maybe_get_constant_statement(&self, id: &Option<ConstId>) -> Option<&AstStatement> {
        id.as_ref().and_then(|it| self.get_constant_statement(it))
    }

    /// query the constants arena for an expression associated with the given `id`
    pub fn get_constant_statement(&self, id: &ConstId) -> Option<&AstStatement> {
        self.find_expression(id)
    }

    /// query the constants arena for an expression that can be evaluated to an i128.
    /// returns an Err if no expression was associated, or the associated expression is a
    /// complex one (not a LiteralInteger)
    pub fn get_constant_int_statement_value(&self, id: &ConstId) -> Result<i128, String> {
        self.get_constant_statement(id)
            .ok_or_else(|| "Cannot find constant expression".into())
            .and_then(|it| match it {
                AstStatement::LiteralInteger { value, .. } => Ok(*value),
                _ => Err(format!("Cannot extract int constant from {:#?}", it)),
            })
    }
}

impl<'a> IntoIterator for &'a ConstExpressions {
    type Item = (ConstId, &'a AstStatement);
    type IntoIter = IntoStatementIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        IntoStatementIter {
            inner: self.expressions.iter(),
        }
    }
}

pub struct IntoStatementIter<'a> {
    inner: Iter<'a, ConstWrapper>,
}

impl<'a> Iterator for IntoStatementIter<'a> {
    type Item = (ConstId, &'a AstStatement);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|(idx, expr)| (idx, expr.get_statement()))
    }
}