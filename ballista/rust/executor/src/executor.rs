// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

//! Ballista executor logic

use std::sync::Arc;

use ballista_core::error::BallistaError;
use ballista_core::execution_plans::ShuffleWriterExec;
use ballista_core::serde::protobuf;
use datafusion::physical_plan::display::DisplayableExecutionPlan;
use datafusion::physical_plan::ExecutionPlan;

/// Ballista executor
pub struct Executor {
    /// Directory for storing partial results
    work_dir: String,
}

impl Executor {
    /// Create a new executor instance
    pub fn new(work_dir: &str) -> Self {
        Self {
            work_dir: work_dir.to_owned(),
        }
    }
}

impl Executor {
    /// Execute one partition of a query stage and persist the result to disk in IPC format. On
    /// success, return a RecordBatch containing metadata about the results, including path
    /// and statistics.
    pub async fn execute_shuffle_write(
        &self,
        job_id: String,
        stage_id: usize,
        part: usize,
        plan: Arc<dyn ExecutionPlan>,
    ) -> Result<Vec<protobuf::ShuffleWritePartition>, BallistaError> {
        // TODO to enable shuffling we need to specify the output partitioning here and
        // until we do that there is always a single output partition
        // see https://github.com/apache/arrow-datafusion/issues/707
        let shuffle_output_partitioning = None;

        let exec = ShuffleWriterExec::try_new(
            job_id,
            stage_id,
            plan,
            self.work_dir.clone(),
            shuffle_output_partitioning,
        )?;
        let partitions = exec.execute_shuffle_write(part).await?;

        println!(
            "=== Physical plan with metrics ===\n{}\n",
            DisplayableExecutionPlan::with_metrics(&exec)
                .indent()
                .to_string()
        );

        Ok(partitions)
    }

    pub fn work_dir(&self) -> &str {
        &self.work_dir
    }
}
