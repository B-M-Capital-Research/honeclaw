/** Hone authenticated, server-owned trade workspace. Financial state remains in memory only. */
export class TradeStoreError extends Error {
  constructor(message, code, status = 0) { super(message); this.name = 'TradeStoreError'; this.code = code; this.status = status; }
}

export function createRemoteStore({ fetchImpl = globalThis.fetch, validate, createEmpty, onInvalidate = () => {}, baseUrl = '/api/public/trade-manager', requestId = () => globalThis.crypto.randomUUID() }) {
  if (typeof fetchImpl !== 'function' || typeof validate !== 'function' || typeof createEmpty !== 'function') throw new Error('交易账户服务配置无效');
  if (!/^\/api\/[A-Za-z0-9/_-]+$/.test(baseUrl)) throw new Error('账户 API 必须使用同源路径');
  let revision = null, context = null, snapshot = null, pending = null, loaded = false, busy = false, blocked = false, epoch = 0;
  const requests = new Set();
  const clone = value => value == null ? value : JSON.parse(JSON.stringify(value));
  const failure = (message, code, status) => new TradeStoreError(message, code, status);
  const checked = state => {
    const result = validate(state);
    if (result !== true && !result?.valid) throw failure(result?.errors?.slice(0, 3).join('；') || '账户数据格式无效', 'INVALID_STATE');
    return clone(state);
  };
  const validContext = value => typeof value === 'string' && value.length > 0 && value.length <= 4096;
  const validRevision = value => Number.isSafeInteger(value) && value >= 0;
  const current = (allowBlocked = false) => {
    if (!loaded || !context || revision === null || !snapshot) throw failure('请先登录 Hone 并成功加载交易账户。', 'NOT_LOADED');
    if (blocked && !allowBlocked) throw failure('账户版本已变化或上次保存结果未确认。请先下载待保存副本，再重新加载；工具不会覆盖服务器数据。', 'RELOAD_REQUIRED', 409);
  };
  function invalidate(reason = '登录状态已变化，请重新打开交易管理台。', cause = 'auth') {
    epoch++;
    for (const controller of requests) controller.abort();
    requests.clear();
    revision = null; context = null; snapshot = null; pending = null; loaded = false; busy = false; blocked = false;
    onInvalidate(reason, cause);
  }
  async function request(path, options = {}, bind = true) {
    const generation = epoch, binding = context, controller = new AbortController();
    requests.add(controller);
    const headers = { Accept: 'application/json', ...(options.body ? { 'Content-Type': 'application/json' } : {}), ...(bind && binding ? { 'X-Hone-Trade-Context': binding } : {}) };
    try {
      const response = await fetchImpl(`${baseUrl}${path}`, { ...options, headers, credentials: 'same-origin', cache: 'no-store', redirect: 'error', signal: controller.signal });
      if (generation !== epoch) throw failure('登录状态已变化，旧请求结果已丢弃。', 'SESSION_CHANGED');
      if ([401, 402, 403].includes(response.status)) {
        invalidate('登录已失效、权限不足或账号已更换。请在 Hone 重新登录后打开工具。');
        throw failure('登录已失效、权限不足或账号已更换，页面中的账户数据已清空。', 'AUTH_REQUIRED', response.status);
      }
      if (!response.ok) {
        if (response.status === 409) throw failure('服务器账户版本已更新。本次修改没有覆盖服务器，请下载待保存副本后重新加载。', 'CONFLICT', 409);
        if (response.status === 413) throw failure('账户数据超过服务器允许的大小，本次修改未保存。', 'TOO_LARGE', 413);
        if (response.status === 400 || response.status === 422) throw failure('服务器未接受本次数据，请检查字段与数据格式。', 'REJECTED', response.status);
        throw failure('服务器暂时无法确认账户操作，请重新加载核对保存结果。', 'SERVER_UNAVAILABLE', response.status);
      }
      let data;
      try { data = await response.json(); }
      catch { throw failure('服务器返回的账户响应无法读取，请重新加载确认。', 'INVALID_RESPONSE', response.status); }
      if (generation !== epoch) throw failure('登录状态已变化，旧请求结果已丢弃。', 'SESSION_CHANGED');
      if (!data || typeof data !== 'object' || !validContext(data.context)) throw failure('服务器缺少会话绑定信息，已停止账户操作。', 'INVALID_RESPONSE');
      if (bind && binding && data.context !== binding) {
        invalidate('账号上下文已变化，旧页面数据已清空。请重新打开交易管理台。');
        throw failure('账号上下文已变化，已拒绝使用其他会话的响应。', 'SESSION_CHANGED');
      }
      return data;
    } catch (error) {
      if (generation !== epoch && !['AUTH_REQUIRED', 'SESSION_CHANGED'].includes(error?.code)) throw failure('登录状态已变化，旧请求结果已丢弃。', 'SESSION_CHANGED');
      if (error instanceof TradeStoreError) throw error;
      throw failure('网络中断，操作结果尚未确认。请重新加载核对；不要重复录入成交。', 'NETWORK_UNCERTAIN');
    } finally { requests.delete(controller); }
  }
  function envelope(data) {
    if (!validRevision(data.revision)) throw failure('服务器账户版本无效，无法继续保存。', 'INVALID_RESPONSE');
    if (data.state === null && data.revision !== 0) throw failure('服务器账户数据缺失，未创建替代账户。', 'INVALID_RESPONSE');
    return { state: checked(data.state === null ? createEmpty() : data.state), revision: data.revision, context: data.context };
  }
  async function load() {
    // Explicit reload is a new lifecycle: never paint any former user's state during it.
    invalidate('正在验证 Hone 登录并加载账户…', 'reload');
    const generation = epoch;
    const data = envelope(await request('/state', { method: 'GET' }, false));
    if (generation !== epoch) throw failure('加载期间登录状态已变化，旧账户数据未显示。', 'SESSION_CHANGED');
    revision = data.revision; context = data.context; snapshot = data.state; loaded = true; blocked = false; pending = null;
    return clone(snapshot);
  }
  async function mutate(path, method, body, candidate = null) {
    current();
    if (busy) throw failure('正在保存上一项修改，请等待完成后再操作。', 'BUSY');
    busy = true;
    const generation = epoch, expectedRevision = revision;
    if (candidate) pending = checked(candidate);
    try {
      const payload = JSON.stringify({ ...body, revision, requestId: requestId() });
      if (new TextEncoder().encode(payload).byteLength > 2 * 1024 * 1024) throw failure('账户数据连同保存信息超过 2 MiB，请减少内容或拆分后再保存。', 'TOO_LARGE', 413);
      const data = envelope(await request(path, { method, body: payload }));
      if (generation !== epoch) throw failure('登录状态已变化，旧保存结果已丢弃。', 'SESSION_CHANGED');
      if (data.revision !== expectedRevision + 1) throw failure('服务器未返回本次保存的新版本，请重新加载确认。', 'INVALID_RESPONSE');
      revision = data.revision; context = data.context; snapshot = data.state; pending = null; blocked = false;
      return clone(snapshot);
    } catch (error) {
      if (generation === epoch && !['REJECTED', 'TOO_LARGE', 'INVALID_STATE', 'BUSY'].includes(error?.code)) blocked = true;
      throw error;
    } finally { if (generation === epoch) busy = false; }
  }
  async function commit(state, label = '保存资料') {
    const candidate = checked(state);
    return mutate('/state', 'PUT', { state: candidate, label: String(label).slice(0, 100) }, candidate);
  }
  async function replace(state, label = '导入备份') { return commit(state, label); }
  async function history() {
    current();
    const generation = epoch;
    const data = await request('/history', { method: 'GET' });
    if (generation !== epoch) throw failure('登录状态已变化，旧恢复记录已丢弃。', 'SESSION_CHANGED');
    if (!Array.isArray(data.history) || data.history.length > 8 || data.history.some(x => !x || !/^[A-Za-z0-9_-]{1,100}$/.test(x.id) || typeof x.id !== 'string' || !validRevision(x.revision) || typeof x.label !== 'string' || typeof x.date !== 'string' || Number.isNaN(Date.parse(x.date)))) throw failure('服务器恢复记录无效，未执行恢复。', 'INVALID_RESPONSE');
    return clone(data.history);
  }
  function snapshotPath(id) {
    if (typeof id !== 'string' || !/^[A-Za-z0-9_-]{1,100}$/.test(id)) throw failure('恢复点编号无效', 'INVALID_STATE');
    return `/history/${encodeURIComponent(id)}`;
  }
  async function getSnapshot(id) {
    current();
    const generation = epoch;
    const data = await request(snapshotPath(id), { method: 'GET' });
    if (generation !== epoch) throw failure('登录状态已变化，旧恢复点已丢弃。', 'SESSION_CHANGED');
    if (data.id !== id || !validRevision(data.revision) || typeof data.label !== 'string' || typeof data.date !== 'string' || Number.isNaN(Date.parse(data.date))) throw failure('恢复点响应无效', 'INVALID_RESPONSE');
    return { id, revision: data.revision, label: data.label, date: data.date, state: checked(data.state) };
  }
  async function recover(id) { return mutate(`${snapshotPath(id)}/restore`, 'POST', {}); }
  async function validateIdentity() {
    // Export remains possible after a version conflict, but only for this same session.
    current(true);
    const generation = epoch;
    await request('/state', { method: 'GET' });
    if (generation !== epoch) throw failure('登录状态已变化，已取消数据导出。', 'SESSION_CHANGED');
    return true;
  }
  async function checkSession() {
    current();
    const expectedRevision = revision, generation = epoch;
    const data = await request('/state', { method: 'GET' });
    if (generation !== epoch) throw failure('登录状态已变化，旧检查结果已丢弃。', 'SESSION_CHANGED');
    if (!validRevision(data.revision)) throw failure('服务器账户版本无效', 'INVALID_RESPONSE');
    if (busy || revision !== expectedRevision) return true;
    if (data.revision !== revision) { blocked = true; throw failure('另一页面已更新账户。请先保存未提交内容的副本，再重新加载。', 'CONFLICT', 409); }
    return true;
  }
  return {
    load, commit, replace, history, getSnapshot, recover, restore: recover, checkSession, validateIdentity, invalidate,
    exportRaw: () => snapshot ? JSON.stringify(snapshot, null, 2) : null,
    getPendingState: () => clone(pending),
    getStatus: () => ({ loaded, saving: busy, blocked, revision }),
  };
}
