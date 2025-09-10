use clap::{Parser, Subcommand};
use env_logger;
use log;
use std::path::PathBuf;
use mz_curve::{ProcessingRequest, process_file, DataLoader};

/// mz_curve - 质谱数据处理工具
#[derive(Parser)]
#[command(name = "mzcurve")]
#[command(about = "质谱数据处理工具，支持DT曲线提取和峰值分析")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// 启用详细日志输出
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// 处理单个文件
    Process {
        /// 输入文件路径
        #[arg(short, long)]
        input: PathBuf,
        
        /// m/z范围 (格式: min-max)
        #[arg(short = 'z', long)]
        mz_range: String,
        
        /// 保留时间范围 (格式: min-max)
        #[arg(short = 't', long)]
        rt_range: String,
        
        /// MS级别
        #[arg(short = 'l', long, default_value = "1")]
        ms_level: u8,
        
        /// 处理模式 (dt, tic, peak)
        #[arg(short, long, default_value = "dt")]
        mode: String,
        
        /// 输出文件路径
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    
    /// 批量处理文件
    Batch {
        /// 输入目录
        #[arg(short, long)]
        input_dir: PathBuf,
        
        /// 输出目录
        #[arg(short, long)]
        output_dir: PathBuf,
        
        /// 配置文件路径
        #[arg(short, long)]
        config: Option<PathBuf>,
    },
    
    /// 验证文件格式
    Validate {
        /// 要验证的文件路径
        #[arg(short, long)]
        file: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    // 初始化日志
    if cli.verbose {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Debug)
            .init();
    } else {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Info)
            .init();
    }
    
    match cli.command {
        Commands::Process { input, mz_range, rt_range, ms_level, mode, output } => {
            process_single_file(input, mz_range, rt_range, ms_level, mode, output).await?;
        }
        Commands::Batch { input_dir, output_dir, config } => {
            process_batch_files(input_dir, output_dir, config).await?;
        }
        Commands::Validate { file } => {
            validate_file(file).await?;
        }
    }
    
    Ok(())
}

async fn process_single_file(
    input: PathBuf,
    mz_range: String,
    rt_range: String,
    ms_level: u8,
    mode: String,
    output: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("开始处理文件: {:?}", input);
    
    let request = ProcessingRequest {
        file_path: input.to_string_lossy().to_string(),
        mz_range,
        rt_range,
        ms_level,
        mode,
    };
    
    let result = process_file(request).await?;
    
    log::info!("处理完成: {} 条曲线, {} 个峰值", result.curve_count(), result.peak_count());
    
    if let Some(output_path) = output {
        // 导出结果
        log::info!("导出结果到: {:?}", output_path);
        // 这里需要实现导出逻辑
    }
    
    Ok(())
}

async fn process_batch_files(
    input_dir: PathBuf,
    output_dir: PathBuf,
    _config: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("批量处理目录: {:?} -> {:?}", input_dir, output_dir);
    
    // 创建输出目录
    std::fs::create_dir_all(&output_dir)?;
    
    // 读取输入目录中的文件
    let entries = std::fs::read_dir(&input_dir)?;
    
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("mzML") {
            log::info!("处理文件: {:?}", path);
            
            // 这里需要实现批量处理逻辑
            // 可以使用配置文件来设置参数
        }
    }
    
    Ok(())
}

async fn validate_file(file: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("验证文件: {:?}", file);
    
    // 尝试加载文件来验证格式
    let container = DataLoader::load_from_file(
        &file.to_string_lossy()
    )?;
    
    log::info!("文件验证成功:");
    log::info!("  - 光谱数量: {}", container.spectra.len());
    log::info!("  - 曲线数量: {}", container.curves.len());
    log::info!("  - 峰值数量: {}", container.peaks.len());
    
    Ok(())
}
