// Copyright 2021 Datafuse Labs
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::collections::BTreeMap;
use std::fmt::Display;
use std::fmt::Formatter;

use crate::ast::write_comma_separated_list;
use crate::ast::write_period_separated_list;
use crate::ast::Hint;
use crate::ast::Identifier;
use crate::ast::Query;

#[derive(Debug, Clone, PartialEq)]
pub struct InsertStmt {
    pub hints: Option<Hint>,
    pub catalog: Option<Identifier>,
    pub database: Option<Identifier>,
    pub table: Identifier,
    pub columns: Vec<Identifier>,
    pub source: InsertSource,
    pub overwrite: bool,
}

impl Display for InsertStmt {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "INSERT ")?;
        if let Some(hints) = &self.hints {
            write!(f, "{} ", hints)?;
        }
        if self.overwrite {
            write!(f, "OVERWRITE ")?;
        } else {
            write!(f, "INTO ")?;
        }
        write_period_separated_list(
            f,
            self.catalog
                .iter()
                .chain(&self.database)
                .chain(Some(&self.table)),
        )?;
        if !self.columns.is_empty() {
            write!(f, " (")?;
            write_comma_separated_list(f, &self.columns)?;
            write!(f, ")")?;
        }
        write!(f, " {}", self.source)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum InsertSource {
    Streaming {
        format: String,
        rest_str: String,
        start: usize,
    },
    StreamingV2 {
        settings: BTreeMap<String, String>,
        start: usize,
    },
    Values {
        rest_str: String,
    },
    Select {
        query: Box<Query>,
    },
}

impl Display for InsertSource {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            InsertSource::Streaming {
                format,
                rest_str,
                start: _,
            } => write!(f, "FORMAT {format} {rest_str}"),
            InsertSource::StreamingV2 { settings, start: _ } => {
                write!(f, " FILE_FORMAT = (")?;
                for (k, v) in settings.iter() {
                    write!(f, " {} = '{}'", k, v)?;
                }
                write!(f, " )")
            }
            InsertSource::Values { rest_str } => write!(f, "VALUES {rest_str}"),
            InsertSource::Select { query } => write!(f, "{query}"),
        }
    }
}
