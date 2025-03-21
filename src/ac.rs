use aho_corasick::AhoCorasick;
use anyhow::Result;
use bincode::config;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;
use tracing::{info, warn};

/// AC自动机封装，用于敏感词匹配
#[derive(Serialize, Deserialize)]
pub struct AcMachine {
    // 存储所有敏感词的集合
    words: Vec<String>,
    // AC自动机实例，使用serde(skip)标记不进行序列化
    #[serde(skip)]
    ac: Option<AhoCorasick>,
}

impl AcMachine {
    /// 创建新的AC自动机实例
    pub fn new() -> Self {
        // 返回一个空的AcMachine实例
        Self {
            words: Vec::new(),
            ac: None,
        }
    }

    /// 从敏感词列表构建AC自动机
    pub fn from_words(words: Vec<String>) -> Self {
        // 创建一个包含敏感词的AcMachine实例
        let mut machine = Self { words, ac: None };
        // 构建AC自动机
        machine.build();
        // 返回构建好的实例
        machine
    }

    /// 构建AC自动机
    pub fn build(&mut self) {
        // 检查敏感词列表是否为空
        if self.words.is_empty() {
            // 如果为空，记录警告并返回
            warn!("No sensitive words to build AC machine");
            return;
        }
        // 记录正在构建AC自动机的信息
        info!("Building AC machine with {} words", self.words.len());
        // 使用词汇列表构建AC自动机
        self.ac = Some(AhoCorasick::new(self.words.clone()).unwrap());
    }

    /// 在文本中查找敏感词
    pub fn find_matches(&self, text: &str) -> Vec<(usize, usize, &str)> {
        // 检查AC自动机是否已构建
        if let Some(ac) = &self.ac {
            // 使用AC自动机查找所有匹配项
            ac.find_iter(text)
                // 将匹配项转换为(开始位置,结束位置,匹配词)的元组
                .map(|mat| (mat.start(), mat.end(), self.words[mat.pattern()].as_str()))
                // 收集所有匹配项到向量中
                .collect()
        } else {
            // 如果AC自动机未构建，记录警告并返回空结果
            warn!("AC machine not built yet");
            Vec::new()
        }
    }

    /// 保存AC机器到文件
    pub async fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        // 将AcMachine实例序列化为二进制数据
        let serialized = bincode::serde::encode_to_vec(&self, config::standard())?;
        // 将序列化数据写入文件
        fs::write(path, serialized).await?;
        // 成功返回
        Ok(())
    }

    /// 从文件加载AC机器
    pub async fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        // 从文件读取序列化数据
        let data = fs::read(path).await?;
        // 将二进制数据反序列化为AcMachine实例
        let (mut machine, _): (AcMachine, usize) =
            bincode::serde::decode_from_slice(&data, config::standard())?;
        // 注释掉的旧版反序列化代码
        // let mut machine: AcMachine = bincode::deserialize(&data)?;
        // 重新构建AC自动机（因为ac字段不会被序列化）
        machine.build();
        // 返回加载好的实例
        Ok(machine)
    }

    /// 过滤文本中的敏感词，用*替换
    pub fn filter_text(&self, text: &str) -> String {
        // 检查AC自动机是否已构建
        if let Some(ac) = &self.ac {
            // 创建文本的可变副本
            let mut filtered = text.to_string();
            // 找出所有匹配项
            let matches: Vec<_> = ac.find_iter(text).collect();

            // 从后向前替换，避免位置偏移
            for mat in matches.iter().rev() {
                // 创建相应长度的*字符串作为替换内容
                let replacement = "*".repeat(mat.end() - mat.start());
                // 替换敏感词
                filtered.replace_range(mat.start()..mat.end(), &replacement);
            }
            // 返回过滤后的文本
            filtered
        } else {
            // 如果AC自动机未构建，记录警告并返回原文本
            warn!("AC machine not built yet");
            text.to_string()
        }
    }
}
