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

//! Defines the EXPLAIN operator

use std::any::Any;
use std::sync::Arc;

use crate::{
    error::{DataFusionError, Result},
    logical_plan::StringifiedPlan,
    physical_plan::Partitioning,
    physical_plan::{common::SizedRecordBatchStream, DisplayFormatType, ExecutionPlan},
};
use arrow::{array::StringBuilder, datatypes::SchemaRef, record_batch::RecordBatch};

use super::SendableRecordBatchStream;
use async_trait::async_trait;

/// Explain execution plan operator. This operator contains the string
/// values of the various plans it has when it is created, and passes
/// them to its output.
#[derive(Debug, Clone)]
pub struct ExplainExec {
    /// The schema that this exec plan node outputs
    schema: SchemaRef,
    /// The strings to be printed
    stringified_plans: Vec<StringifiedPlan>,
    /// control which plans to print
    verbose: bool,
}

impl ExplainExec {
    /// Create a new ExplainExec
    pub fn new(
        schema: SchemaRef,
        stringified_plans: Vec<StringifiedPlan>,
        verbose: bool,
    ) -> Self {
        ExplainExec {
            schema,
            stringified_plans,
            verbose,
        }
    }

    /// The strings to be printed
    pub fn stringified_plans(&self) -> &[StringifiedPlan] {
        &self.stringified_plans
    }
}

#[async_trait]
impl ExecutionPlan for ExplainExec {
    /// Return a reference to Any that can be used for downcasting
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn schema(&self) -> SchemaRef {
        self.schema.clone()
    }

    fn children(&self) -> Vec<Arc<dyn ExecutionPlan>> {
        // this is a leaf node and has no children
        vec![]
    }

    /// Get the output partitioning of this plan
    fn output_partitioning(&self) -> Partitioning {
        Partitioning::UnknownPartitioning(1)
    }

    fn with_new_children(
        &self,
        children: Vec<Arc<dyn ExecutionPlan>>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        if children.is_empty() {
            Ok(Arc::new(self.clone()))
        } else {
            Err(DataFusionError::Internal(format!(
                "Children cannot be replaced in {:?}",
                self
            )))
        }
    }

    async fn execute(&self, partition: usize) -> Result<SendableRecordBatchStream> {
        if 0 != partition {
            return Err(DataFusionError::Internal(format!(
                "ExplainExec invalid partition {}",
                partition
            )));
        }

        let mut type_builder = StringBuilder::new(self.stringified_plans.len());
        let mut plan_builder = StringBuilder::new(self.stringified_plans.len());

        let plans_to_print = self
            .stringified_plans
            .iter()
            .filter(|s| s.should_display(self.verbose));

        for p in plans_to_print {
            type_builder.append_value(p.plan_type.to_string())?;
            plan_builder.append_value(&*p.plan)?;
        }

        let record_batch = RecordBatch::try_new(
            self.schema.clone(),
            vec![
                Arc::new(type_builder.finish()),
                Arc::new(plan_builder.finish()),
            ],
        )?;

        Ok(Box::pin(SizedRecordBatchStream::new(
            self.schema.clone(),
            vec![Arc::new(record_batch)],
        )))
    }

    fn fmt_as(
        &self,
        t: DisplayFormatType,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        match t {
            DisplayFormatType::Default => {
                write!(f, "ExplainExec")
            }
        }
    }
}
