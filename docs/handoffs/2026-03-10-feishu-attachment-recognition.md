# 飞书附件识别修复 handoff

最后更新：2026-03-10
状态：已完成

## 背景

- 用户反馈：飞书里上传图片后，下游助手提示收到 `image...bin`，无法按图片解析

## 根因

- `bridges/hone-feishu-facade/main.go` 在处理 `image` 消息时，兜底文件名是 `image_<key>.bin`
- facade 下载附件后只透传了 `filename/local_path/size`，没有透传 `content_type`
- Rust 侧 `infer_attachment_kind()` 依赖 MIME 或文件后缀识别类型，因此把该附件归类成 `Other`

## 修复

- 在 facade 下载阶段读取响应头 `Content-Type`
- 当响应头为空或仅为 `application/octet-stream` 时，用文件前 512 字节做内容嗅探
- 根据最终 MIME 给 `.bin/.dat/.tmp` 这类兜底文件名补真实扩展名
- 将 `content_type` 一并透传给 Rust

## 验证

- 执行：`cd bridges/hone-feishu-facade && go test ./...`
- 结果：通过

## 影响范围

- 直接影响 Feishu 渠道的图片/PDF 等附件分类
- 不涉及消息发送协议、会话存储和其他渠道
