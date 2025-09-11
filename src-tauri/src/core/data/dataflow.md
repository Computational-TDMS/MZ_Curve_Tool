# DataContainer 结构说明

我们所遵循的原则是,一个container对应一个数据

一个数据当然可以提取出多条曲线,而多条曲线则对应多个识别出的峰

通过这样树状的结构,我们就能够存储好所有的中间数据了

## 当前的数据结构

```
DataContainer
    ---metadata (HashMap<String, serde_json::Value>)
    ---spectrums (Vec<Spectrum>)
    ---curves (Vec<Curve>)
        ---peaks (Vec<Peak>)  # 嵌套在 Curve 中

```

## 内存管理优化

当前的问题是,这样的datacontainer最后可能会很大,我们已经实现了以下优化措施:

1. **缓存功能**: 提供了 `remove_processed_spectra` 方法来清理已处理的数据
2. **分块处理功能**: 实现了 `split_chunks` 方法将大数据集分割成小块处理
3. **内存使用估算**: 添加了 `estimate_memory_usage` 方法来监控内存消耗

## 性能考虑

Datacontainer的结构体的大小是动态的，但通过以下方式减少性能损耗:

1. 使用 `#[serde(skip)]` 属性避免序列化大型光谱数据
2. 实现了内存估算和清理机制
3. 提供了分块处理功能避免一次性加载所有数据到内存