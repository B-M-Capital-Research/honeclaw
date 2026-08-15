# Bug: 单行评级归属矛盾——action 与所附文章互相冲突仍升 High 即时

## 发现时间

- 2026-08-15 20:10 CST（两周重放实弹演示的逐条 review 中发现）

## Bug Type

- Data Quality / False Alert

## 严重等级

- P2

## 状态

- Fixed（2026-08-15）

## 现象

2026-08-03 09:44 UTC，GEV 收到一条 High 即时推送「Morgan Stanley 下调至
Underweight（原 Overweight）」，但其 `newsURL`/`newsTitle` 指向的文章是
《William Blair Adds GE Vernova (GEV) to Conviction List》——一篇看多文。
FMP `v4/upgrades-downgrades` 在**单行级别**把 action 字段与证据文章错配，
是 2026-08-14 MU「多股汇总文扇出」事故的更窄变种：单行、单公司文章，
汇总文防线（标题模式 / 同 URL ≥3 行）不覆盖。

## 影响面

- 近 14 天窗口扫描：67 条单行评级中 4 条存在「行内署名券商 ≠ 标题券商」
  矛盾（GEV/Morgan Stanley、CRWV/UBS、MU/Seaport、SNDK/UBS），其中仅
  GEV 一条是 downgrade 升到了 High 即时，其余为 hold 仅入摘要。
- 连带:「近 30 日共识计数」（item 3）聚合 store 历史单行时，会把
  2026-08-14 前入库的汇总文污染行与此类归属矛盾行计入下调数;汇总摘要
  事件的 `counts` 本身也是跨股错配嫌疑数据，不应进统计口径。

## 修复记录

- `is_title_attribution_suspect(firm, title, action)`:标题未署名该券商
  （含 BofA/JPMorgan/RBC 缩写别名，剔除 Securities/Capital 等通用后缀词）
  → 矛盾；署名在场但 action=downgrade 且标题只有正面动作词
  （conviction list / PT raised / top pick）且无任何下调词 → 矛盾。
  空标题不判（无证据不算矛盾）。
- 命中且原判 High 的 downgrade → 降 Medium，summary 追加
  「（署名券商未见于原文标题，归属存疑，请以原文核实）」，payload 写
  `hone_grade_attribution_suspect: true`。
- 共识计数改为**只信干净单行**：跳过汇总文标题的历史污染行、跳过归属
  存疑行、不再计入汇总摘要事件的 `counts`。
- 验收：GEV 真实案例单测（Medium + 存疑标注 + payload 标记）；
  真实下调（署名在标题内，含 BofA 缩写形态）保持 High 不误伤；
  两周重放 dry-run 中该条从即时（63 条）移入摘要（62 条），其余不变；
  events.jsonl 离线回放评级 High 7 → 6，全量 637 测试通过。

## 证据来源

- `data/events.jsonl` `grade:GEV:2026-08-03T09:44:00.000Z:Morgan Stanley`
  （payload.newsTitle 与 action 互斥）
- 重放审计脚本对 14 天窗口的归属扫描（4/67 矛盾行清单见会话记录）
