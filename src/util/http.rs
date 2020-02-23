//! HTTP 相关工具
#![allow(dead_code)]
use std::collections::HashMap;

/// 从 Url 中的查询字符串构造的 HashMap Wrapper
#[derive(Debug)]
pub struct QueryString<'a>(HashMap<&'a str, &'a str>);

impl<'a> QueryString<'a> {
    /// 从 Url 的查询字符串构造一个 QueryString 对象
    pub fn new(s: &'a str) -> Self {
        Self::from(s)
    }
    /// 查询某个键
    pub fn get(&self, s: &str) -> Option<&'a str> {
        self.0.get(s).copied()
    }
    /// 判断是否包含某个键
    pub fn contains(&self, s: &str) -> bool {
        self.0.contains_key(s)
    }
    /// 获取键值对个数
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a> From<&'a str> for QueryString<'a> {
    fn from(s: &'a str) -> Self {
        let vec = s.split('&').collect::<Vec<&str>>();
        let mut map = HashMap::with_capacity(vec.len());
        for pair in vec {
            if let Some(index) = pair.find('=') {
                map.insert(&pair[0..index], &pair[index + 1..]);
            }
        }
        Self(map)
    }
}
