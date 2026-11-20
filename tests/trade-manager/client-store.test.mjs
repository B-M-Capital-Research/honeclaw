import test from 'node:test';
import assert from 'node:assert/strict';
import {createRemoteStore} from '../../packages/app/public/trade-manager/client-store.js';

const CONTEXT='opaque-session-context';
const state=(cash=0)=>({settings:{cash},holdings:[],plans:[]});
const envelope=(value=state(),revision=1,context=CONTEXT)=>({state:value,revision,context});
const response=(body,status=200)=>({status,ok:status>=200&&status<300,json:async()=>body});
const deferred=()=>{let resolve,reject;const promise=new Promise((yes,no)=>{resolve=yes;reject=no;});return{promise,resolve,reject};};
function setup(replies){
  const calls=[],invalidations=[];
  const store=createRemoteStore({
    fetchImpl:async(url,options)=>{calls.push({url,options});const next=replies.shift();if(next instanceof Error)throw next;return typeof next==='function'?next(url,options):next;},
    validate:value=>({valid:!!value&&Array.isArray(value.holdings)&&Number.isFinite(value.settings?.cash)}),
    createEmpty:()=>state(),onInvalidate:(message,cause)=>invalidations.push({message,cause}),
    requestId:()=> 'abcb349a-0100-4000-9000-b41139a27d51',
  });
  return{store,calls,invalidations};
}
const rejectsCode=(promise,code)=>assert.rejects(promise,error=>error.code===code);

test('only an authenticated null revision-zero response creates a blank workspace',async()=>{
  const {store,calls,invalidations}=setup([response(envelope(null,0))]);
  assert.deepEqual(await store.load(),state());
  assert.deepEqual(invalidations.map(x=>x.cause),['reload']);
  assert.equal(store.getStatus().revision,0);
  assert.equal(calls[0].options.headers['X-Hone-Trade-Context'],undefined);
  assert.equal(calls[0].options.credentials,'same-origin');
  assert.equal(calls[0].options.cache,'no-store');
  assert.equal(calls[0].options.redirect,'error');
});

test('load failure and malformed data never create sample or fallback financial state',async()=>{
  for(const reply of [new Error('offline'),response(envelope(null,2)),response({state:state(),revision:1}),response(envelope({bad:true}))]){
    const {store}=setup([reply]);
    await assert.rejects(store.load());
    assert.equal(store.exportRaw(),null);
    assert.equal(store.getStatus().loaded,false);
    await rejectsCode(store.commit(state(42)),'NOT_LOADED');
  }
});

test('commit sends session binding and revision without a client user id, publishes only saved canonical state',async()=>{
  const saved=deferred();const {store,calls}=setup([response(envelope(state(10),3)),()=>saved.promise]);
  await store.load();const candidate=state(20);const operation=store.commit(candidate,'录入入金');
  candidate.settings.cash=999;
  assert.equal(JSON.parse(store.exportRaw()).settings.cash,10);
  assert.equal(store.getPendingState().settings.cash,20);
  assert.equal(store.getStatus().saving,true);
  await rejectsCode(store.commit(state(30)),'BUSY');
  const body=JSON.parse(calls[1].options.body);
  assert.equal(body.revision,3);assert.equal(body.label,'录入入金');
  assert.equal(body.userId,undefined);assert.equal(body.state.settings.cash,20);
  assert.equal(calls[1].options.headers['X-Hone-Trade-Context'],CONTEXT);
  saved.resolve(response(envelope(state(21),4)));
  assert.deepEqual(await operation,state(21));
  assert.equal(JSON.parse(store.exportRaw()).settings.cash,21);
  assert.equal(store.getPendingState(),null);assert.equal(store.getStatus().saving,false);
});

test('conflicts preserve a downloadable candidate and reject further writes until explicit reload',async()=>{
  const {store,calls}=setup([response(envelope(state(10),2)),response({},409),response(envelope(state(40),3))]);
  await store.load();await rejectsCode(store.commit(state(20)),'CONFLICT');
  assert.equal(JSON.parse(store.exportRaw()).settings.cash,10);
  const copy=store.getPendingState();copy.settings.cash=999;
  assert.equal(store.getPendingState().settings.cash,20);
  await rejectsCode(store.replace(state(90)),'RELOAD_REQUIRED');
  assert.equal(calls.length,2);
  assert.deepEqual(await store.load(),state(40));
  assert.equal(store.getPendingState(),null);
});

test('ambiguous network or server failures retain old state and block accidental duplicate records',async()=>{
  for(const reply of [new Error('connection lost'),response({},500),{status:200,ok:true,json:async()=>{throw Error('invalid JSON');}}]){
    const {store}=setup([response(envelope(state(10),1)),reply]);await store.load();
    await assert.rejects(store.commit(state(20)));
    assert.equal(JSON.parse(store.exportRaw()).settings.cash,10);
    assert.equal(store.getPendingState().settings.cash,20);
    assert.equal(store.getStatus().blocked,true);
    await rejectsCode(store.commit(state(20)),'RELOAD_REQUIRED');
  }
});

test('a success without the next revision is not accepted as a saved update',async()=>{
  for(const revision of [2,4]){
    const {store}=setup([response(envelope(state(10),2)),response(envelope(state(20),revision))]);
    await store.load();await rejectsCode(store.commit(state(20)),'INVALID_RESPONSE');
    assert.equal(JSON.parse(store.exportRaw()).settings.cash,10);
    assert.equal(store.getStatus().blocked,true);
  }
});

test('401, 402 and 403 immediately erase state, context and unsaved candidate',async()=>{
  for(const status of [401,402,403]){
    const {store,invalidations}=setup([response(envelope(state(10))),response({},status)]);
    await store.load();await rejectsCode(store.commit(state(20)),'AUTH_REQUIRED');
    assert.equal(store.exportRaw(),null);assert.equal(store.getPendingState(),null);
    assert.equal(store.getStatus().loaded,false);assert.equal(store.getStatus().saving,false);
    assert.deepEqual(invalidations.map(x=>x.cause),['reload','auth']);
  }
});

test('context mismatch during reads erases the previous account and rejects the other session response',async()=>{
  const {store,invalidations}=setup([response(envelope(state(10))),response({history:[],context:'different-session'})]);
  await store.load();await rejectsCode(store.history(),'SESSION_CHANGED');
  assert.deepEqual(invalidations.map(x=>x.cause),['reload','auth']);
  assert.equal(store.exportRaw(),null);assert.equal(store.getStatus().loaded,false);
});

test('logout aborts inflight writes and a late response cannot restore financial state',async()=>{
  const late=deferred();const {store,calls}=setup([response(envelope(state(10))),()=>late.promise]);
  await store.load();const operation=store.commit(state(20));
  store.invalidate('logout');assert.equal(calls[1].options.signal.aborted,true);
  late.resolve(response(envelope(state(20),2)));
  await rejectsCode(operation,'SESSION_CHANGED');
  assert.equal(store.exportRaw(),null);assert.equal(store.getPendingState(),null);
});

test('identity invalidation while parsing JSON cannot publish an old account response',async()=>{
  const parsed=deferred();const {store}=setup([{status:200,ok:true,json:()=>parsed.promise}]);
  const operation=store.load();await Promise.resolve();await Promise.resolve();
  store.invalidate('different user');parsed.resolve(envelope(state(100)));
  await rejectsCode(operation,'SESSION_CHANGED');assert.equal(store.exportRaw(),null);
});

test('a new login is isolated from a former load even when abort is ignored',async()=>{
  const first=deferred();const {store}=setup([()=>first.promise,response(envelope(state(200),5,'new-session'))]);
  const old=store.load();const fresh=store.load();
  assert.deepEqual(await fresh,state(200));
  first.resolve(response(envelope(state(100),1,'old-session')));
  await rejectsCode(old,'SESSION_CHANGED');
  assert.equal(JSON.parse(store.exportRaw()).settings.cash,200);
  assert.equal(store.getStatus().revision,5);
});

test('history lookup is context bound and server restore uses the current revision',async()=>{
  const snapshot={id:'snapshot_1',date:'2026-11-16T00:00:00Z',label:'资金基线',revision:1};
  const {store,calls}=setup([
    response(envelope(state(80),4)),response({history:[snapshot],context:CONTEXT}),
    response({...snapshot,state:state(10),context:CONTEXT}),response(envelope(state(10),5)),
  ]);
  await store.load();assert.deepEqual(await store.history(),[snapshot]);
  assert.equal((await store.getSnapshot(snapshot.id)).state.settings.cash,10);
  assert.equal(JSON.parse(store.exportRaw()).settings.cash,80);
  assert.deepEqual(await store.recover(snapshot.id),state(10));
  assert.equal(calls[3].url,'/api/public/trade-manager/history/snapshot_1/restore');
  assert.equal(JSON.parse(calls[3].options.body).revision,4);
  for(const call of calls.slice(1))assert.equal(call.options.headers['X-Hone-Trade-Context'],CONTEXT);
});

test('external revision change is detected without replacing displayed account',async()=>{
  const {store}=setup([response(envelope(state(10),2)),response(envelope(state(30),3))]);
  await store.load();await rejectsCode(store.checkSession(),'CONFLICT');
  assert.equal(JSON.parse(store.exportRaw()).settings.cash,10);
  assert.equal(store.getStatus().blocked,true);
});

test('a session check begun before a completed save cannot create a false conflict',async()=>{
  const check=deferred();const {store}=setup([response(envelope(state(10),2)),()=>check.promise,response(envelope(state(20),3))]);
  await store.load();const checking=store.checkSession();await store.commit(state(20));
  check.resolve(response(envelope(state(10),2)));assert.equal(await checking,true);
  assert.equal(store.getStatus().blocked,false);assert.equal(store.getStatus().revision,3);
});

test('serialized UTF-8 request size is checked before sending and includes envelope overhead',async()=>{
  const {store,calls}=setup([response(envelope(state(10),2))]);await store.load();
  const huge=state(20);huge.note='金'.repeat(710000);
  await rejectsCode(store.commit(huge),'TOO_LARGE');
  assert.equal(calls.length,1);assert.equal(JSON.parse(store.exportRaw()).settings.cash,10);
  assert.equal(store.getStatus().saving,false);assert.equal(store.getStatus().blocked,false);
});

test('identity validation permits downloading a conflicted candidate without accepting newer state',async()=>{
  const {store,calls}=setup([response(envelope(state(10),1)),response({},409),response(envelope(state(99),8))]);
  await store.load();await rejectsCode(store.commit(state(20)),'CONFLICT');
  assert.equal(await store.validateIdentity(),true);
  assert.equal(calls[2].options.headers['X-Hone-Trade-Context'],CONTEXT);
  assert.equal(store.getStatus().blocked,true);assert.equal(store.getStatus().revision,1);
  assert.equal(store.getPendingState().settings.cash,20);
  assert.equal(JSON.parse(store.exportRaw()).settings.cash,10);
});

test('identity validation on export clears pending data if the account has changed',async()=>{
  const {store}=setup([response(envelope(state(10),1)),response({},409),response(envelope(state(99),8,'another-context'))]);
  await store.load();await rejectsCode(store.commit(state(20)),'CONFLICT');
  await rejectsCode(store.validateIdentity(),'SESSION_CHANGED');
  assert.equal(store.exportRaw(),null);assert.equal(store.getPendingState(),null);
});

test('rejected validation leaves the prior server state intact while a corrected submission may proceed',async()=>{
  const {store}=setup([response(envelope(state(10),1)),response({},422),response(envelope(state(30),2))]);
  await store.load();await rejectsCode(store.commit(state(20)),'REJECTED');
  assert.equal(store.getStatus().blocked,false);
  assert.equal(JSON.parse(store.exportRaw()).settings.cash,10);
  assert.deepEqual(await store.commit(state(30)),state(30));
});
