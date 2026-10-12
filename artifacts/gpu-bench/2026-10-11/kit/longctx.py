# usage: longctx.py URL KEY MODEL OUT.jsonl CTX_TOKENS MINUTES N1,N2,...
# Each of max(N) sessions gets its own ~CTX-token history (warmed once). Each round, N sessions append
# their last reply + a fresh short question (append-only, like an agent), stream 512 tokens; logs TTFT, per-stream decode, aggregate.
import json,sys,time,threading,random,urllib.request
url,key,model,out,ctx,mins,Ns=sys.argv[1],sys.argv[2],sys.argv[3],sys.argv[4],int(sys.argv[5]),float(sys.argv[6]),[int(x) for x in sys.argv[7].split(',')]
W="system memory cache kernel thread buffer packet router switch latency throughput compiler parser token vector matrix tensor gradient optimizer scheduler queue lock mutex atomic page frame socket stream file block disk volume shard replica leader follower quorum commit rollback index query plan join scan filter sort merge hash tree graph node edge weight bias layer head attention decoder encoder".split()
def history(s):
    r=random.Random(s); words=int(ctx*0.72)
    body=" ".join(r.choice(W)+("." if r.random()<0.07 else "") for _ in range(words))
    return [{"role":"system","content":"You are a helpful engineering assistant."},{"role":"user","content":f"Session {s} design notes:\n"+body},{"role":"assistant","content":"Noted. I have read the design notes."}]
H={}
def call(s,q,max_tokens,res):
    msgs=H[s]+[{"role":"user","content":q}]
    text=[]
    body=json.dumps({"model":model,"messages":msgs,"max_tokens":max_tokens,"ignore_eos":True,"temperature":0.7,"stream":True,"stream_options":{"include_usage":True}}).encode()
    r=urllib.request.Request(url+"/v1/chat/completions",body,{"Content-Type":"application/json","Authorization":"Bearer "+key})
    t0=time.time(); first=None; n=0; ptok=None
    try:
        for line in urllib.request.urlopen(r,timeout=1800):
            line=line.strip()
            if not line.startswith(b"data:") or line==b"data: [DONE]": continue
            j=json.loads(line[5:])
            if j.get("usage"): n=j["usage"].get("completion_tokens",n); ptok=j["usage"].get("prompt_tokens")
            for c in j.get("choices",[]):
                d=c.get("delta",{})
                if d.get("content") or d.get("reasoning_content") or d.get("reasoning"):
                    if first is None: first=time.time()
                if d.get("content"): text.append(d["content"])
        H[s]=msgs+[{"role":"assistant","content":"".join(text) or "OK."}]
        t1=time.time(); res[s]=dict(ttft=(first or t1)-t0,decode=(n-1)/(t1-first) if first and t1>first and n>1 else 0,tok=n,ptok=ptok,end=t1)
    except Exception as e: res[s]=dict(err=str(e)); print("err",s,e,flush=True)
S=max(Ns)
for s in range(S): H[s]=history(s)
print("warming",S,"sessions",flush=True)
res={}; th=[threading.Thread(target=call,args=(s,"Summarize in one word.",1,res)) for s in range(S)]
t=time.time(); [x.start() for x in th]; [x.join() for x in th]
print("warm done %.0fs"%(time.time()-t), {s:res[s].get("ptok") for s in res},flush=True)
end=time.time()+mins*60; k=0
with open(out,"a") as f:
    while time.time()<end:
        for n in Ns:
            k+=1; res={}
            th=[threading.Thread(target=call,args=(s,f"Question {k}: describe one risk in these notes.",512,res)) for s in range(n)]
            t=time.time(); [x.start() for x in th]; [x.join() for x in th]
            ok=[v for v in res.values() if "ttft" in v]
            if not ok: continue
            w=max(v["end"] for v in ok)-t
            f.write(json.dumps({"ts":time.time(),"n":n,"ctx":ok[0]["ptok"],"agg":sum(v["tok"] for v in ok)/w,"ttft":[v["ttft"] for v in ok],"per":[v["decode"] for v in ok]})+"\n"); f.flush()
