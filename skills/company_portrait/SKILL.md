---
name: Company Portrait
description: 维护长期公司画像、投资主线 与事件时间线；当用户想建立公司档案、在财报/公告后更新长期判断，或希望保留结论、为什么成立、关键证据与研究路径时使用
when_to_use: 当用户正在系统研究一家公司，明确希望长期跟踪、沉淀画像、维护 投资主线，或在财报/公告/管理层变化后把新结论写回长期档案时使用
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
- 当前 actor 用户空间下的 `company_profiles/<profile_id>/events/*.md`：事件驱动的 投资主线变更日志

`<profile_id>` **必须是 lowercase 的美股 ticker**（双股票的公司用 `googl-goog` 这种连字符形式），不要用公司名（不要写 `circle`、`ciena`、`rocket-lab` 这种）。下游 投资主线 蒸馏靠 ticker 把画像和持仓对齐，目录名跑偏会让画像被悄悄忽略。

`profile.md` **首行必须是 YAML frontmatter**，至少包含 `ticker` 字段：

```markdown
---
ticker: GOOGL
---

# Alphabet (GOOGL)

...
```

双 ticker 公司写 `ticker: GOOGL / GOOG`。frontmatter 是下游识别的唯一可靠路径——标题随便写都行（`# Alphabet`、`# GOOGL`、`# Alphabet 母公司画像` 都允许），但 frontmatter 缺失会让这份画像在 投资主线 蒸馏 / global digest 里彻底失效。

当前运行时还没有独立 `research_notes` 存储层。需要保留“为什么这么判断、证据、当时怎么搜的”时，先写进事件正文与 `refs`，不要假装系统里已经存在单独研究笔记文件。

默认语言跟随当前用户对话语言；除非用户明确要求，或引用材料必须保留原文，否则不要无故切换语言。

## 何时触发

- 用户正在系统研究一家公司，而不是只问一句短问题
- 用户明确说希望长期跟踪、沉淀画像、建立档案、记录 投资主线
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
   - 只要用户是在发起新的系统性公司调研，就默认建立长期画像并沉淀本轮结论
   - 不需要再额外征求一次“是否建档”的确认
   - 仅当用户明显只是在问一个一次性短问题，或明确表示这轮不要沉淀时，才不要创建

3. 需要建档时，直接在用户空间创建画像目录和 Markdown：

```text
company_profiles/<profile_id>/profile.md
company_profiles/<profile_id>/events/
```

4. 若画像已存在：
   - 先直接读取现有 `profile.md`
   - 用已有画像和最近事件作为本轮研究上下文
   - 如果只是重复已有结论，不要写任何更新
   - 若本轮沉淀了值得长期复用的内容，应主动回写，不要等用户逐条要求
   - 若本轮已经改变长期判断、稳定偏好、共识逻辑或估值结论，应优先直接改 `profile.md` 正文或对应 section，不要只在事件里补一条流水账

5. 若出现净新增事实，再直接追加事件文件：

```text
company_profiles/<profile_id>/events/<date>-<event_type>-<slug>.md
```

6. 若长期判断已经变化明显，再直接回写主画像对应 section。

## 严格规则

- **profile_id 必须是 lowercase 美股 ticker**（双 ticker 用 `googl-goog`）；profile.md 顶部必须有 `ticker:` frontmatter，否则下游 投资主线 蒸馏直接忽略这份画像
- 用户发起新的系统性公司调研时，默认沉淀到长期画像；除非用户明确拒绝，否则不要因为缺少二次确认而跳过建档
- 只要本轮研究产出了值得长期复用的内容，就应主动记录到画像或事件，尤其是用户自己的看法、偏好或约束、你们此前已经达成一致的判断逻辑，以及本轮估值判断、估值区间或估值锚点
- 所有建档、改写、追加事件，都应优先通过 runner 原生文件读写完成
- 主画像记录“当前最佳判断”，不要把历史流水账堆进 `profile.md`
- 不要把事件区当成日记本；该改正文就改正文，只有确实存在净新增事实或 投资主线 change 时才追加事件
- 事件只记录净新增事实与长期判断变化，不写空泛评论
- 每个值得长期保留的判断，尽量说明为什么成立、看了什么、什么事实会推翻它
- 默认使用用户当前对话语言维护画像与事件；如需保留原始英文术语、标题或引用，可局部保留原文
- 不要把日内价格波动写进长期画像
- 画像是长期研究资产，不是交易建议清单；不要输出直接买卖指令
