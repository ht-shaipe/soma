//! Feature trait 定义与 TypedFeature 强类型桥接
//!
//! 两种实现方式：
//! 1. 实现 [`TypedFeature`]（推荐）：输入输出为 serde 结构体，派生 `JsonSchema`
//!    后由 blanket impl 自动桥接为 [`Feature`]，Schema 自动生成；
//! 2. 直接实现 [`Feature`]：完全掌控 JSON 信封（适合动态/代理型功能点）。

use crate::context::FeatureContext;
use crate::descriptor::{FeatureDescriptor, FeatureMeta};
use crate::envelope::FeatureInput;
use crate::progress::ProgressReporter;
use serde::de::DeserializeOwned;
use serde::Serialize;
use soma_core::error::SomaError;
use std::sync::Arc;

/// 功能点抽象：每个可独立调用的能力实现此 trait
///
/// - 输入为 [`FeatureInput`] 信封（payload 为该功能点约定的 JSON 结构）
/// - 输出为 JSON 值（结构由 output schema 约定），由注册表运行器包进
///   [`FeatureOutput`](crate::FeatureOutput) 信封
/// - 实现应把产物文件写入 `ctx.work_dir()` 并通过
///   [`add_artifact`](crate::FeatureContext::add_artifact) 登记
pub trait Feature: Send + Sync {
    /// 功能点描述（含输入输出 Schema，用于 features/list）
    fn descriptor(&self) -> FeatureDescriptor;

    /// 执行功能点
    ///
    /// 返回 `Err` 即执行失败（运行器会记录为 Failed 并持久化运行记录）。
    fn run(
        &self,
        ctx: &mut FeatureContext,
        input: &FeatureInput,
        progress: &dyn ProgressReporter,
    ) -> Result<serde_json::Value, SomaError>;
}

/// 强类型功能点：输入输出为 serde 结构体（"serde 定义即 Schema"）
///
/// 输入输出类型派生 `schemars::JsonSchema` 后，
/// `features/list` 会自动携带 JSON Schema，为插件化/第三方调用铺路。
pub trait TypedFeature: Send + Sync {
    /// 入参类型
    type Input: DeserializeOwned + schemars::JsonSchema;
    /// 出参类型
    type Output: Serialize + schemars::JsonSchema;

    /// 功能点元信息（id/名称/描述/分类）
    fn meta(&self) -> FeatureMeta;

    /// 执行功能点
    fn run(
        &self,
        ctx: &mut FeatureContext,
        input: Self::Input,
        progress: &dyn ProgressReporter,
    ) -> Result<Self::Output, SomaError>;
}

/// TypedFeature → Feature 的统一桥接：入参反序列化校验、出参序列化、Schema 生成
impl<T> Feature for T
where
    T: TypedFeature,
{
    fn descriptor(&self) -> FeatureDescriptor {
        let meta = self.meta();
        let input_schema = serde_json::to_value(schemars::schema_for!(T::Input)).ok();
        let output_schema = serde_json::to_value(schemars::schema_for!(T::Output)).ok();
        FeatureDescriptor {
            id: meta.id,
            name: meta.name,
            description: meta.description,
            kind: meta.kind,
            input_schema,
            output_schema,
        }
    }

    fn run(
        &self,
        ctx: &mut FeatureContext,
        input: &FeatureInput,
        progress: &dyn ProgressReporter,
    ) -> Result<serde_json::Value, SomaError> {
        let typed: T::Input = serde_json::from_value(input.payload.clone())?;
        let output = TypedFeature::run(self, ctx, typed, progress)?;
        Ok(serde_json::to_value(output)?)
    }
}

/// 便捷构造：把具体功能点实例包装为注册表所需的 trait 对象
pub fn shared<F>(feature: F) -> Arc<dyn Feature>
where
    F: Feature + 'static,
{
    Arc::new(feature)
}
