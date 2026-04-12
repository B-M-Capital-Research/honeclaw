---
name: Company Portrait
description: 维护长期公司画像、thesis 与事件时间线；当用户想建立公司档案、在财报/公告后更新长期判断，或希望保留结论、为什么成立、关键证据与研究路径时使用
when_to_use: 当用户正在系统研究一家公司，明确希望长期跟踪、沉淀画像、维护 thesis，或在财报/公告/管理层变化后把新结论写回长期档案时使用
user-invocable: true
context: inline
aliases:
  - company portrait
  - 公司画像
  - 长期画像
---

## Company Portrait

把单次研究升级成可持续维护的研究档案。

当前实现采用文档型存储：

- 当前 actor 用户空间下的 `company_profiles/<profile_id>/profile.md`：当前仍然成立的主画像，不写流水账
- 当前 actor 用户空间下的 `company_profiles/<profile_id>/events/*.md`：事件驱动的 thesis change log

当前运行时还没有独立 `research_notes` 存储层。需要保留“为什么这么判断、证据、当时怎么搜的”时，先写进事件正文与 `refs`，不要假装系统里已经存在单独研究笔记文件。

## 何时触发

- 用户正在系统研究一家公司，而不是只问一句短问题
- 用户明确说希望长期跟踪、沉淀画像、建立档案、记录 thesis
- 用户在查看财报、公告、管理层变化后，希望把新信息追加到已有画像
- 用户希望下次继续研究时，系统还能记得“当前结论、为什么成立、证伪条件、研究路径”

## 先读哪些参考

- 需要写首版画像或大改主画像时，先读 [references/profile-framework.md](references/profile-framework.md)
- 需要追加事件时，先读 [references/event-template.md](references/event-template.md)
- 需要保留“为什么”和“当时怎么搜的”时，先读 [references/research-trail.md](references/research-trail.md)

## 工作流

1. 先用原生文件读写能力检查画像是否存在：

```bash
rg --files company_profiles
```

2. 如果还没有画像：
   - 必须先询问用户是否要建立长期画像
   - 不要自动创建

3. 用户确认后，再直接在用户空间创建画像目录和 Markdown：

```text
company_profiles/<profile_id>/profile.md
company_profiles/<profile_id>/events/
```

4. 若画像已存在：
   - 先直接读取现有 `profile.md`
   - 用已有画像和最近事件作为本轮研究上下文
   - 如果只是重复已有结论，不要写任何更新

5. 若出现净新增事实，再直接追加事件文件：

```text
company_profiles/<profile_id>/events/<date>-<event_type>-<slug>.md
```

6. 若长期判断已经变化明显，再直接回写主画像对应 section。

## 严格规则

- 未经用户确认，不得首次创建画像
- 所有建档、改写、追加事件，都应优先通过 runner 原生文件读写完成
- 主画像记录“当前最佳判断”，不要把历史流水账堆进 `profile.md`
- 事件只记录净新增事实与长期判断变化，不写空泛评论
- 每个值得长期保留的判断，尽量说明为什么成立、看了什么、什么事实会推翻它
- 不要把日内价格波动写进长期画像
- 画像是长期研究资产，不是交易建议清单；不要输出直接买卖指令
