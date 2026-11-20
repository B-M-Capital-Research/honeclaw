import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import vm from 'node:vm';

// Execute the real child cleanup path with a small DOM harness. This checks
// that server auth invalidation reaches the parent, while normal loads do not.
const app=readFileSync(new URL('../../packages/app/public/trade-manager/app.js',import.meta.url),'utf8');
const cleanup=app.slice(app.indexOf('function clearSensitive('),app.indexOf('function renderLocked('));
function harness(){
  const posts=[],nodes=new Map();
  const node=id=>{if(!nodes.has(id))nodes.set(id,{open:false,close(){this.open=false;},replaceChildren(){},value:'',hidden:false,textContent:''});return nodes.get(id);};
  const context=vm.createContext({
    lifecycle:0,loading:false,state:{private:'old-account'},uiSaving:true,checkingSession:true,
    mainInitial:new Map([['form','private']]),quotePreview:{},search:'private',eventSearch:'private',
    reportFrom:'2026-01-01',reportTo:'2026-02-01',storageError:'old',authMessage:'',modalInitial:'private',toastTimer:1,
    clearDownload(){},clearTimeout(){},renderLocked(){},syncHostStatus(){},
    $:node,window:{},parent:{postMessage:(data,origin)=>posts.push({data,origin})},location:{origin:'https://hone.test'},
  });
  vm.runInContext(cleanup,context);
  return{context,posts};
}

test('server auth cleanup sends a no-data parent invalidation and clears child memory',()=>{
  const {context,posts}=harness();
  context.clearSensitive('arbitrary translated message','auth');
  assert.equal(context.state,null);assert.equal(context.mainInitial.size,0);assert.equal(context.loading,false);
  assert.equal(posts.length,1);
  assert.equal(posts[0].origin,'https://hone.test');
  assert.equal(JSON.stringify(posts[0].data),JSON.stringify({type:'hone:trade-auth-invalidated'}));
});

test('ordinary loading never sends auth failure, regardless of the display text',()=>{
  const {context,posts}=harness();
  context.clearSensitive('login expired is just display text here','reload');
  assert.equal(context.state,null);assert.equal(context.loading,true);assert.equal(posts.length,0);
  context.parent=context.window;
  context.clearSensitive('standalone page auth failure','auth');
  assert.equal(posts.length,0);
});
