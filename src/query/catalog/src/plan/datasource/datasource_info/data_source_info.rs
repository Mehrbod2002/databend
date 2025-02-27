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

use std::sync::Arc;

use common_expression::TableSchema;
use common_meta_app::schema::TableInfo;

use crate::plan::ParquetTableInfo;
use crate::plan::ResultScanTableInfo;
use crate::plan::StageTableInfo;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum DataSourceInfo {
    // Normal table source, `fuse/system`.
    TableSource(TableInfo),
    // Internal/External source, like `s3://` or `azblob://`.
    StageSource(StageTableInfo),
    // stage source with parquet format used for select.
    ParquetSource(ParquetTableInfo),
    // Table Function Result_Scan
    ResultScanSource(ResultScanTableInfo),
}

impl DataSourceInfo {
    pub fn schema(&self) -> Arc<TableSchema> {
        match self {
            DataSourceInfo::TableSource(table_info) => table_info.schema(),
            DataSourceInfo::StageSource(table_info) => table_info.schema(),
            DataSourceInfo::ParquetSource(table_info) => table_info.schema(),
            DataSourceInfo::ResultScanSource(table_info) => table_info.schema(),
        }
    }

    pub fn desc(&self) -> String {
        match self {
            DataSourceInfo::TableSource(table_info) => table_info.desc.clone(),
            DataSourceInfo::StageSource(table_info) => table_info.desc(),
            DataSourceInfo::ParquetSource(table_info) => table_info.desc(),
            DataSourceInfo::ResultScanSource(table_info) => table_info.desc(),
        }
    }
}
