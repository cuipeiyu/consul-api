# consul-api

<p align="center">
  <a href="https://github.com/cuipeiyu/consul-api">
    <img src="https://img.shields.io/badge/Rust-1.75+-dea584.svg" alt="Rust Version" />
  </a>
  <a href="https://github.com/cuipeiyu/consul-api/blob/main/LICENSE-MIT">
    <img src="https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg" alt="License" />
  </a>
  <a href="https://crates.io/crates/consul-api">
    <img src="https://img.shields.io/crates/v/consul-api.svg" alt="Crates.io" />
  </a>
  <a href="https://docs.rs/consul-api">
    <img src="https://docs.rs/consul-api/badge.svg" alt="Documentation" />
  </a>
</p>

<p align="center">
  <strong>🦀 A Rust client library for HashiCorp Consul API</strong>
</p>

<p align="center">
  <a href="#功能特性">功能特性</a> ·
  <a href="#安装">安装</a> ·
  <a href="#快速开始">快速开始</a> ·
  <a href="#API-文档">API 文档</a> ·
  <a href="#测试">测试</a> ·
  <a href="#贡献">贡献</a>
</p>

---

## 📖 简介

`consul-api` 是一个用 Rust 编写的 HashiCorp Consul API 客户端库，提供类型安全、异步的 API 调用方式。支持 Consul 的所有核心功能，包括服务发现、健康检查、KV 存储等。

> **⚠️ 开发状态说明**
> 
> 本项目当前版本为 **`0.0.7-pre`**，处于 **积极开发阶段**。
> 
> - 🔧 API 接口可能会发生变化，使用时请注意版本更新
> - 📝 文档持续完善中，部分功能可能缺少示例
> - 🧪 已在本地 Consul 环境通过集成测试（31 个测试）
> - ⚡ 建议用于开发/测试环境，生产环境请谨慎评估
> 
> 欢迎提交 Issue 和 PR 帮助改进项目！

### ✨ 特性

- ✅ **类型安全** - 使用 Rust 的强类型系统，编译时捕获错误
- ✅ **异步支持** - 基于 `tokio` 和 `reqwest`，支持高并发
- ✅ **完整覆盖** - 覆盖 Consul API 的所有核心功能
- ✅ **自动生成** - 使用代码生成工具自动生成结构体定义
- ✅ **易于使用** - 提供 Builder 模式，简化客户端配置

---

## 🚀 安装

在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
consul-api = "0.0.7-pre"

# 或者使用 git 版本
[dependencies]
consul-api = { git = "https://github.com/cuipeiyu/consul-api", tag = "0.0.7-pre" }
```

### 要求

- Rust 1.75+
- Consul 1.20+ (支持 1.20.x 和 1.22.x)

---

## 🎯 快速开始

### 创建客户端

```rust
use consul_api::Client;

// 最简单的方式
let client = Client::new();

// 使用 Builder 模式自定义配置
let client = Client::builder()
    .address("http://127.0.0.1:8500")
    .timeout(std::time::Duration::from_secs(30))
    .build()
    .expect("Failed to create client");
```

### 从环境变量读取配置

```rust
use consul_api::Client;

// 自动读取 CONSUL_HTTP_ADDR 和 CONSUL_HTTP_TOKEN
let client = Client::from_env();
```

---

## 📚 API 文档

### 1️⃣ Agent API

Agent API 用于与本地 Consul agent 交互，管理服务和检查。

#### 注册服务

```rust
use consul_api::*;

let client = Client::new();

// 定义服务
let service = ServiceDefinition {
    id: "my-service-1".to_string(),
    name: "my-service".to_string(),
    tags: vec!["v1".to_string(), "api".to_string()],
    address: "127.0.0.1".to_string(),
    port: 8080,
    ..Default::default()
};

// 注册服务
let result = client.agent_register_service(&Default::default(), &service).await;
assert!(result.is_ok());
```

#### 注册健康检查

```rust
// 注册 TTL 检查
let check = CheckDefinition {
    id: "my-check".to_string(),
    name: "My TTL Check".to_string(),
    ttl: Some("60s".to_string()),
    ..Default::default()
};

let query = RegisterCheckRequestQuery {
    check: check.clone(),
    ..Default::default()
};
client.agent_register_check(&query).await?;

// 更新 TTL 检查状态
let query = AgentTTLCheckRequestQuery {
    check_id: "my-check".to_string(),
    note: Some("Service is healthy".to_string()),
    ..Default::default()
};
client.agent_check_pass(&query).await?;
```

#### 查询服务和检查

```rust
// 获取所有注册的服务
let services = client.agent_services().await?;
for (id, service) in services {
    println!("Service: {}, Address: {}", id, service.address);
}

// 获取所有检查
let checks = client.agent_checks().await?;
for (id, check) in checks {
    println!("Check: {}, Status: {}", id, check.status);
}
```

---

### 2️⃣ KV Store API

KV Store API 用于操作 Consul 的键值存储。

#### 创建/更新键值

```rust
let client = Client::new();

// 创建键值
let key = "my-app/config";
let value = b"{\"host\": \"localhost\", \"port\": 8080}";

let result = client.kv_create_or_update_key(key, value.to_vec(), &Default::default()).await;
assert!(result.is_ok());
assert_eq!(result.unwrap(), true);
```

#### 读取键值

```rust
// 读取键值（返回 JSON 格式）
let result = client.kv_read_key(key, &Default::default()).await;
if let Ok(Some(response)) = result {
    // response 是 Vec<KVPair>
    for pair in response {
        println!("Key: {}, Value: {}", pair.key,
                  String::from_utf8_lossy(&pair.value));
    }
}

// 读取原始值
let mut query = KVReadKeyQuery::default();
query.raw = Some(true);
let result = client.kv_read_key(key, &query).await;
if let Ok(Some(value)) = result {
    println!("Raw value: {}", String::from_utf8_lossy(&value));
}
```

#### 删除键值

```rust
let result = client.kv_delete_key(key, &Default::default()).await;
assert!(result.is_ok());
assert_eq!(result.unwrap(), true);
```

---

### 3️⃣ Catalog API

Catalog API 用于查询和注册 datacenter 级别的元数据。

#### 列出数据中心

```rust
let client = Client::new();

let datacenters = client.catalog_list_datacenters().await?;
for dc in datacenters {
    println!("Datacenter: {}", dc);
}
```

#### 列出节点和服务

```rust
// 列出所有节点
let nodes = client.catalog_list_nodes().await?;
for node in nodes {
    println!("Node: {}, Address: {}", node.node, node.address);
}

// 列出所有服务
let services = client.catalog_list_services().await?;
for (name, tags) in services {
    println!("Service: {}, Tags: {:?}", name, tags);
}

// 列出服务的所有节点
let query = CatalogNodesForServiceRequestQuery::default();
let nodes = client.catalog_list_nodes_for_service("my-service", &query).await?;
for node in nodes {
    println!("Node: {}, ServicePort: {:?}", node.node, node.service_port);
}
```

#### 查询节点的服务

```rust
let services = client.catalog_node_services("my-node", &Default::default()).await?;
if let Some(services) {
    for service in services.services {
        println!("Service: {}", service.service_name);
    }
}
```

---

### 4️⃣ Health API

Health API 用于查询健康状态和过滤结果。

#### 查询健康节点

```rust
let client = Client::new();

// 查询所有健康的节点
let nodes = client.health_list_nodes("web", &Default::default()).await?;
for node in nodes {
    println!("Node: {}, Status: {}", node.node.node, node.checks[0].status);
}
```

#### 查询健康服务

```rust
// 查询所有健康的服务实例
let instances = client.health_list_service_instances("web", &Default::default()).await?;
for instance in instances {
    println!("Service: {}, Node: {}",
             instance.service.unwrap().service_name,
             instance.node.node);
}
```

#### 按状态查询

```rust
// 查询所有 passing 状态的检查
let checks = client.health_list_state(Health::Passing, &Default::default()).await?;
for check in checks {
    println!("Check: {}, Status: {}", check.check_id, check.status);
}
```

---

### 5️⃣ Event API

Event API 用于触发和查询 Consul 事件。

#### 触发事件

```rust
let client = Client::new();

let event_name = "deploy";
let event_payload = b"{\"version\": \"1.0.0\"}";

let result = client.event_fire(event_name, event_payload.to_vec(), &Default::default()).await;
assert!(result.is_ok());
```

#### 查询事件

```rust
let events = client.event_list(&Default::default()).await?;
for event in events {
    println!("Event: {}, Payload: {:?}", event.name, event.payload);
}
```

---

### 6️⃣ Status API

Status API 用于查询 Consul 集群状态。

#### 查询 Leader 和 Peers

```rust
let client = Client::new();

// 查询 Raft leader
let leader = client.status_leader().await?;
println!("Leader: {}", leader);

// 查询 Raft peers
let peers = client.status_peers().await?;
for peer in peers {
    println!("Peer: {}", peer);
}
```

---

## 🧪 测试

### 运行测试

```bash
# 确保 Consul 正在运行
consul agent -dev -bind=127.0.0.1

# 运行所有测试
cargo test

# 运行集成测试
cargo test --test api_integration_test

# 运行单个测试
cargo test --test api_integration_test -- test_kv_operations
```

### 测试覆盖率

当前测试覆盖了 **31 个 API 方法**，覆盖率约 **94%**（不包括企业版功能）。

---

## 🛠️ 项目结构

```
consul-api/
├── src/
│   ├── lib.rs              # 主库文件，包含所有 API 方法
│   ├── structs_1_20_x.rs  # Consul 1.20.x 的结构体定义
│   └── structs_1_22_x.rs  # Consul 1.22.x 的结构体定义
├── scanner/
│   ├── main.go            # 代码生成工具（Go 编写）
│   └── head.rs.tpl       # 结构体模板文件
├── tests/
│   └── api_integration_test.rs  # 集成测试
├── consul-1.20.1/        # Consul 1.20.1 源码
├── consul-1.22.5/        # Consul 1.22.5 源码
└── README.md
```

---

## 🤝 贡献

欢迎贡献！请遵循以下步骤：

1. Fork 本项目
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add some amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 开启 Pull Request

### 代码生成

本项目使用 `scanner` 工具自动生成结构体定义：

```bash
cd scanner
go run main.go
```

---

## 📄 许可证

本项目采用双重许可证：

- **MIT License** - 查看 [LICENSE-MIT](LICENSE-MIT)
- **Apache License 2.0** - 查看 [LICENSE-APACHE](LICENSE-APACHE)

你可以选择其中一种许可证使用本项目。

---

## 🙏 致谢

- [HashiCorp Consul](https://www.consul.io/) - 优秀的 Service Mesh 解决方案
- [reqwest](https://github.com/seanmonstar/reqwest) - 强大的 HTTP 客户端
- [tokio](https://tokio.rs/) - Rust 异步运行时

---

## 📧 联系方式

- **GitHub**: [@cuipeiyu](https://github.com/cuipeiyu)
- **Issues**: [GitHub Issues](https://github.com/cuipeiyu/consul-api/issues)

---

<p align="center">
  ⭐ 如果这个项目对你有帮助，请给它一个 Star！
</p>
