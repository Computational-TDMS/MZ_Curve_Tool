# Spectrum Types in mzdata

因为spectrum我们是直接调用mzdata官方的spectrum结构,基于其原始读取的，
因此仅在这里创建一个说明文档。

## RawSpectrum

表示一个尚未处理的光谱，只包含数据数组，可能没有离散的峰。原始光谱可能仍被质心化，但峰仍需要解码。

### 字段

- `description: SpectrumDescription` - 光谱元数据，描述获取条件和详细信息
- `arrays: BinaryArrayMap` - 描述m/z、强度和潜在其他测量属性的数据数组

### 主要方法

- `new(description: SpectrumDescription, arrays: BinaryArrayMap) -> Self` - 创建新的 RawSpectrum 实例
- `into_centroid<C>(self) -> Result<CentroidSpectrumType<C>, SpectrumConversionError>` - 将光谱转换为 CentroidSpectrumType
- `mzs(&self) -> Cow<[f64]>` - 访问 m/z 数组
- `intensities(&self) -> Cow<[f32]>` - 访问强度数组
- `decode_all_arrays(&mut self) -> Result<(), ArrayRetrievalError>` - 显式解码所有编码或压缩的 DataArray
- `into_spectrum<C, D>(self) -> Result<MultiLayerSpectrum<C, D>, SpectrumConversionError>` - 将光谱转换为 MultiLayerSpectrum
- `denoise(&mut self, scale: f32) -> Result<(), SpectrumProcessingError>` - 对信号应用局部去噪算法
- `pick_peaks_with_into<C, D>(self, peak_picker: &PeakPicker) -> Result<MultiLayerSpectrum<C, D>, SpectrumProcessingError>` - 使用指定的 peak_picker 选择峰值并转换光谱
- `pick_peaks_into<C, D>(self, signal_to_noise_threshold: f32) -> Result<MultiLayerSpectrum<C, D>, SpectrumProcessingError>` - 使用最小信噪比阈值选择峰值并转换光谱

## Spectrum

Spectrum 是 MultiLayerSpectrum<CentroidPeak, DeconvolutedPeak> 的类型别名，表示一个多层重叠的光谱。

### 字段

- `description: SpectrumDescription` - 光谱元数据，描述获取条件和详细信息
- `arrays: Option<BinaryArrayMap>` - (可能不存在的)描述m/z、强度和潜在其他测量属性的数据数组
- `peaks: Option<PeakSetVec<CentroidPeak, MZ>>` - 质心峰集合
- `deconvoluted_peaks: Option<PeakSetVec<DeconvolutedPeak, Mass>>` - 解卷积峰集合


---> 之后可以干脆用来存储一些简单的数据处理模式!

