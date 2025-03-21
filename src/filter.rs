use crate::ac::AcMachine;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use tracing::{error, info};

// 敏感词过滤器结构体
pub struct SensitiveFilter {
    // 使用Arc和RwLock包装AC自动机，支持并发读写
    ac_machine: Arc<RwLock<AcMachine>>,
    // 模型文件目录路径
    models_dir: PathBuf,
    // 敏感词源文件目录路径
    source_dir: PathBuf,
}

impl SensitiveFilter {
    // 创建新的敏感词过滤器实例
    pub async fn new(base_dir: impl AsRef<Path>) -> Result<Self> {
        // 获取基础目录引用
        let base_dir = base_dir.as_ref();
        // 构建模型目录路径
        let models_dir = base_dir.join("models");
        // 构建源文件目录路径
        let source_dir = models_dir.join("source");

        // 确保目录存在
        if !models_dir.exists() {
            // 创建模型目录
            fs::create_dir_all(&models_dir).await?;
        }
        if !source_dir.exists() {
            // 创建源文件目录
            fs::create_dir_all(&source_dir).await?;
        }

        // 创建空的AC自动机并包装为Arc<RwLock>
        let ac_machine = Arc::new(RwLock::new(AcMachine::new()));
        
        // 返回过滤器实例
        Ok(Self {
            ac_machine,
            models_dir,
            source_dir,
        })
    }

    /// 初始化敏感词过滤器
    pub async fn init(&self) -> Result<()> {
        // 构建索引文件路径
        let index_path = self.models_dir.join("ac_index.bin");
        
        // 检查索引文件是否存在
        if index_path.exists() {
            // 记录加载索引信息
            info!("Loading existing AC index from {:?}", index_path);
            // 尝试从文件加载AC自动机
            match AcMachine::load_from_file(&index_path).await {
                Ok(machine) => {
                    // 获取AC自动机的写锁
                    let mut ac = self.ac_machine.write().await;
                    // 更新AC自动机
                    *ac = machine;
                    // 记录加载成功信息
                    info!("Successfully loaded AC index");
                    return Ok(());
                }
                Err(e) => {
                    // 记录加载失败信息
                    error!("Failed to load AC index: {:?}, will build from source", e);
                }
            }
        }

        // 如果索引不存在或加载失败，从源文件重建
        self.rebuild_index().await
    }

    /// 从源文件重建索引
    pub async fn rebuild_index(&self) -> Result<()> {
        // 构建字典文件路径
        let dic_path = self.source_dir.join("dic.txt");
        
        // 检查字典文件是否存在
        if !dic_path.exists() {
            // 记录文件不存在的错误
            error!("Dictionary file not found at {:?}", dic_path);
            return Err(anyhow::anyhow!("Dictionary file not found"));
        }

        // 记录开始构建索引的信息
        info!("Building AC index from {:?}", dic_path);
        // 读取字典文件内容
        let content = fs::read_to_string(&dic_path).await?;
        // 解析字典文件，提取敏感词列表
        let words: Vec<String> = content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.trim().to_string())
            .collect();

        // 检查敏感词列表是否为空
        if words.is_empty() {
            return Err(anyhow::anyhow!("No words found in dictionary file"));
        }

        // 记录从字典中加载的词数
        info!("Loaded {} words from dictionary", words.len());
        // 使用敏感词列表创建AC自动机
        let machine = AcMachine::from_words(words);
        
        // 构建索引文件路径
        let index_path = self.models_dir.join("ac_index.bin");
        // 保存AC自动机到文件
        machine.save_to_file(&index_path).await?;
        // 记录保存成功信息
        info!("Saved AC index to {:?}", index_path);

        // 获取AC自动机的写锁
        let mut ac = self.ac_machine.write().await;
        // 更新内存中的AC自动机
        *ac = machine;

        Ok(())
    }

    /// 过滤文本中的敏感词
    pub async fn filter(&self, text: &str) -> String {
        // 获取AC自动机的读锁
        let ac = self.ac_machine.read().await;
        // 使用AC自动机过滤文本
        ac.filter_text(text)
    }

    /// 查找文本中的敏感词
    pub async fn find_sensitive_words(&self, text: &str) -> Vec<String> {
        // 获取AC自动机的读锁
        let ac = self.ac_machine.read().await;
        // 使用AC自动机查找敏感词
        ac.find_matches(text)
            .into_iter()
            // 只保留匹配到的词，丢弃位置信息
            .map(|(_, _, word)| word.to_string())
            // 收集到向量中
            .collect()
    }
}
