use std::fmt;

use crate::context::Context;
use crate::stmt::Stmt;

#[derive(Debug, PartialEq)]
pub struct File {
    library_name: String,
    imports: Vec<String>,
    pub(crate) stmts: Vec<Stmt>,
}

impl File {
    pub fn new(library_name: &str, context: &Context<'_>) -> Self {
        Self {
            library_name: context.get_library_alias(library_name.to_string()),
            imports: context
                .libraries
                .get(library_name)
                .expect("library exists on config")
                .imports
                .clone(),
            stmts: Vec::new(),
        }
    }

    pub fn add_stmt(&mut self, stmt: Stmt) {
        self.stmts.push(stmt);
    }

    pub fn compare(&self, other: &Self) {
        super::compare_slice(&self.stmts, &other.stmts, |i, self_stmt, other_stmt| {
            let _span = debug_span!("stmt", i).entered();
            self_stmt.compare(other_stmt);
        });
    }
}

impl fmt::Display for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "//! This file has been automatically generated by `objc2`'s `header-translator`."
        )?;
        writeln!(f, "//! DO NOT EDIT")?;

        writeln!(f, "use crate::common::*;")?;
        writeln!(f, "use crate::{}::*;", self.library_name)?;
        for import in &self.imports {
            writeln!(f, "use crate::{import}::*;")?;
        }

        writeln!(f)?;

        for stmt in &self.stmts {
            writeln!(f, "{stmt}")?;
        }

        Ok(())
    }
}
