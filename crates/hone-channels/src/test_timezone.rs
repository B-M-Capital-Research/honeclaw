//! 测试专用：钉住运行时时区。
/// 把运行时时区钉成北京，供依赖"本地墙钟 → 美东交易日"换算的测试使用。
///
/// 运行时时区改造之后 `runtime_timezone()` 会回退到**宿主时区**，而这些断言
/// 是按北京时间写的（北京 08-04 09:31 = 美东 08-03 21:31）。不钉住的话
/// 本地（Asia/Shanghai）能过、CI（UTC）必挂 —— 2026-08-16 就是这么红的。
///
/// 时区是进程级全局，所以这里只允许钉成同一个值；若将来有测试需要别的时区，
/// 必须改成把时区显式传进被测函数，而不是在这里加第二个取值。
pub(crate) fn pin_beijing_runtime_timezone() {
    let _ = hone_core::configure_runtime_timezone(Some("Asia/Shanghai"));
}
