use async_trait::async_trait;
use serde_json::Value;
use crate::core::data::{DataContainer, ProcessingResult, ProcessingError};
use crate::core::processors::dt_extractor::DTExtractor;
use crate::core::processors::baseline_correction::BaselineProcessor;

/// 简化的处理器trait
#[async_trait]
pub trait Processor: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn config_schema(&self) -> Value;
    
    async fn process(
        &self,
        input: DataContainer,
        config: Value,
    ) -> Result<ProcessingResult, ProcessingError>;
}

/// 处理器枚举，用于解决dyn兼容性问题
#[derive(Debug)]
pub enum ProcessorEnum {
    DTExtractor(DTExtractor),
    BaselineProcessor(BaselineProcessor),
    // 可以添加更多处理器类型
}

#[async_trait]
impl Processor for ProcessorEnum {
    fn name(&self) -> &str {
        match self {
            ProcessorEnum::DTExtractor(p) => p.name(),
            ProcessorEnum::BaselineProcessor(p) => p.name(),
        }
    }

    fn description(&self) -> &str {
        match self {
            ProcessorEnum::DTExtractor(p) => p.description(),
            ProcessorEnum::BaselineProcessor(p) => p.description(),
        }
    }

    fn config_schema(&self) -> Value {
        match self {
            ProcessorEnum::DTExtractor(p) => p.config_schema(),
            ProcessorEnum::BaselineProcessor(p) => p.config_schema(),
        }
    }

    async fn process(
        &self,
        input: DataContainer,
        config: Value,
    ) -> Result<ProcessingResult, ProcessingError> {
        match self {
            ProcessorEnum::DTExtractor(p) => p.process(input, config).await,
            ProcessorEnum::BaselineProcessor(p) => p.process(input, config).await,
        }
    }
}

/// 导出器trait
#[async_trait]
pub trait Exporter: Send + Sync {
    fn name(&self) -> &str;
    fn output_format(&self) -> &str;
    
    async fn export(
        &self,
        data: &DataContainer,
        config: Value,
    ) -> Result<String, ProcessingError>;
}
